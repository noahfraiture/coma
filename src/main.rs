use std::{
    error, process,
    sync::{Arc, Mutex},
};

use colored::Colorize;
use tokio::{sync::Semaphore, task::JoinSet};

mod browser;
mod cli;
mod config;
mod extract;
mod format;
mod graph;
mod node;
mod state;

use browser::Browser;
use config::Config;
use node::Node;
use state::State;

static PERMITS: Semaphore = Semaphore::const_new(0);

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()?;
    let mut state = State::new(Arc::clone(&config.root))?;
    println!("{:?}", config.root.lock().unwrap().url);
    PERMITS.add_permits(config.args.task as usize);
    while state.pop_layer().is_some() {
        println!("=== Depth {} ===", state.current_depth);

        let mut handles = browse_layer(&mut state, &config).await?;
        let childs = parse_layer(&mut state, &config, &mut handles).await?;
        state.add_to_next_layer(childs);
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

type FuturesBrowse = JoinSet<(Result<Browser, browser::BrowseError>, Arc<Mutex<Node>>)>;

// Browse the current layer and generate the chromium browser for the page
async fn browse_layer(
    state: &mut State,
    config: &Config,
) -> Result<FuturesBrowse, Box<dyn error::Error>> {
    let mut handles: FuturesBrowse = JoinSet::new();
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

// Parse every page of the layer and extract useful information
async fn parse_layer(
    state: &mut State,
    config: &Config,
    handles: &mut FuturesBrowse,
) -> Result<Vec<Arc<Mutex<Node>>>, Box<dyn std::error::Error>> {
    println!("Collecting data from every url of the layer");
    let mut total_count = 0;
    let mut next_layer_childs = Vec::new();
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

        let mut childs: Vec<Arc<Mutex<Node>>> = links
            .map(|url| Node::new_arc(Some(&parent), url.clone(), url.to_string()))
            .collect();
        total_count += parent.lock().unwrap().quantity_elements() + childs.len();
        next_layer_childs.append(&mut childs);
    }
    println!(
        "Found a total of {} {:?}",
        total_count.to_string().green(),
        config.args.cmd
    );
    Ok(next_layer_childs)
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
