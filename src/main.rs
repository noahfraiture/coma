use std::{process, sync::Arc};

use crate::scrapy::Browser;
use cli::Commands;
use colored::Colorize;
use scrapy::ScrapyError;
use tokio::{sync::Semaphore, task::JoinSet};

mod cli;
mod config;
mod graph;
mod scrapy;
mod state;
mod topology;

use config::Config;
use state::State;
use topology::Node;

static PERMITS: Semaphore = Semaphore::const_new(0);

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()?;
    let mut state = State::new(Arc::clone(&config.root))?;
    println!("{:?}", config.root.url);
    PERMITS.add_permits(config.thread as usize);
    while state.pop_layer().is_some() {
        println!("=== Depth {} ===", state.current_depth);

        let mut handles = browse_layer(&mut state, &config).await?;
        collect(&mut state, &config, &mut handles).await?;
        if state.current_depth == config.target_depth {
            break;
        }
        state.deeper();
        println!();
    }

    if config.cmd == Commands::Graph {
        graph::render(&config.root).await?;
    }
    Ok(())
}

async fn browse_layer(
    state: &mut State,
    config: &Config,
) -> Result<JoinSet<(Result<Browser, ScrapyError>, Arc<Node>)>, Box<dyn std::error::Error>> {
    let mut handles = JoinSet::new();
    while let Some(node) = state.next_url()? {
        if node.url.domain().is_none()
            || !config.same_domain(&node.url)
            || state.known(&node)
            || !config.in_bound(&node.url)
        {
            continue;
        }

        let permit = PERMITS.acquire().await?;
        println!("Visiting {}", node.url.as_str().green());
        handles.spawn(async move {
            let _permit = permit;
            (Browser::new_navigate(&node.url), node)
        });
    }
    Ok(handles)
}

async fn collect(
    state: &mut State,
    config: &Config,
    handles: &mut JoinSet<(Result<Browser, ScrapyError>, Arc<Node>)>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Collecting data from every url of the layer");
    let mut total_count = 0;
    while let Some(handle) = handles.join_next().await {
        let (browser, parent) = handle?;
        let links = browser?.parse_document(config.cmd, &parent).await;
        total_count += parent.quantity_elements() + links.len();

        let childs = links
            .into_iter()
            .map(|url| Node::new_arc(Some(&parent), url.clone(), url.to_string()))
            .collect();
        state.add_to_next_layer(childs)?;
    }
    println!(
        "Found a total of {} {:?}",
        total_count.to_string().green(),
        config.cmd
    );
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
