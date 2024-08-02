use std::process;

use tokio::{sync::Semaphore, task::JoinSet};

use crate::scrapy::Browser;
use colored::Colorize;

mod cli;
mod graph;
mod scrapy;
mod state;

static PERMITS: Semaphore = Semaphore::const_new(0);

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = state::State::new()?;
    PERMITS.add_permits(state.thread as usize);
    while state.pop_layer().is_some() {
        println!("=== Depth {} ===", state.current_depth);
        let mut handles = JoinSet::new();

        // Connect
        while let Some(url) = state.next_url()? {
            // TODO : check that url has a host (is not an ip address or an email)
            if url.domain().is_none()
                || !state.same_domain(&url)
                || state.known(&url)
                || !state.in_bound(&url)
            {
                continue;
            }

            let permit = PERMITS.acquire().await?;
            println!("Visiting {}", url.as_str().green());
            handles.spawn(async move {
                let _permit = permit;
                (Browser::new_navigate(&url), url)
            });
        }

        // Collect result
        println!("Collecting data from every url of the layer");
        let mut total_count = 0;
        while let Some(handle) = handles.join_next().await {
            let (browser, url) = handle?;
            let (links, count) = browser?.parse_document(state.cmd, &url).await;
            total_count += count;
            state.add_to_next_layer(links)?;
        }
        println!(
            "Found a total of {} {:?}",
            total_count.to_string().green(),
            state.cmd
        );
        if state.bottom() {
            break;
        }
        state.deeper();
        println!();
    }
    Ok(())
}

fn main() {
    if let Ok(rt) = tokio::runtime::Runtime::new() {
        if let Err(e) = rt.block_on(run()) {
            eprintln!("{:?}", e);
            process::exit(1);
        }
        return;
    }
    eprintln!("Error: can't start the tokio runtime");
    process::exit(2);
}
