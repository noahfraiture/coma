use colored::Colorize;
use tokio::task::JoinSet;
use chrono::Local;

use crate::scrapy::Browser;

mod cli;
mod scrapy;
mod state;

async fn run() {
    let mut state = state::State::new();
    while state.pop_layer().is_some() {
        let mut handles = JoinSet::new();
        println!("=== New layer ===");

        // Connect
        while let Some(url) = state.next_url() {
            // TODO: remove unwrap
            if !state.same_domain(&url).unwrap() || state.known(&url) {
                continue;
            }

            println!("Visiting {}", url.as_str().green());
            handles.spawn(async {
                Browser::new_navigate(url)
            });

        }

        // Collect result
        while let Some(res) = handles.join_next().await {
            println!("Time : {}", Local::now());
            let (browser, url) = res.unwrap().unwrap();
            let links = browser.parse_document(state.cmd, &url).await;
            state.add_to_next_layer(links);
        }
        if state.bottom() {
            break;
        }
        state.deeper();
        println!();
    }
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(run());
}
