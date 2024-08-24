use std::{
    process,
    sync::{Arc, Mutex},
};

use crate::scrapy::Browser;
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
    println!("{:?}", config.root.lock().unwrap().url);
    PERMITS.add_permits(config.args.task as usize);
    while state.pop_layer().is_some() {
        println!("=== Depth {} ===", state.current_depth);

        let mut handles = browse_layer(&mut state, &config).await?;
        collect(&mut state, &config, &mut handles).await?;
        if state.current_depth == config.args.depth {
            break;
        }
        state.current_depth += 1;
        println!();
    }

    // TODO : format data and build the response
    // graph::render(&config.root).await?;
    Ok(())
}

type FuturesBrowse = JoinSet<(Result<Browser, ScrapyError>, Arc<Mutex<Node>>)>;

async fn browse_layer(
    state: &mut State,
    config: &Config,
) -> Result<FuturesBrowse, Box<dyn std::error::Error>> {
    let mut handles = JoinSet::new();
    while let Some(node) = state.current_layer.pop() {
        if !config.same_domain(&node.lock().unwrap().url)
            || state.known(&node)
            || !config.in_bound(&node.lock().unwrap().url)
        {
            continue;
        }

        let permit = PERMITS.acquire().await?;
        println!("Visiting {}", node.lock().unwrap().url.as_str().green());
        handles.spawn(async move {
            let _permit = permit;
            let url = node.lock().unwrap().url.clone();
            (Browser::new_navigate(&url), node)
        });
    }
    Ok(handles)
}

async fn collect(
    state: &mut State,
    config: &Config,
    handles: &mut FuturesBrowse,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Collecting data from every url of the layer");
    let mut total_count = 0;
    while let Some(handle) = handles.join_next().await {
        let (browser, parent) = handle?;
        let mut explore_external = false;
        let links = browser?
            .parse_document(&config.args.content, &parent)
            .await
            .into_iter()
            .filter(|link| {
                if config.same_domain(link) {
                    return true;
                }
                if state.current_external < config.args.external {
                    explore_external = true;
                    return true;
                }
                false
            });
        parent.lock().unwrap().explored = true;

        let childs: Vec<Arc<Mutex<Node>>> = links
            .map(|url| Node::new_arc(Some(&parent), url.clone(), url.to_string()))
            .collect();
        total_count += parent.lock().unwrap().quantity_elements() + childs.len();
        state.add_to_next_layer(childs);
    }
    println!(
        "Found a total of {} {:?}",
        total_count.to_string().green(),
        config.args.cmd
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
