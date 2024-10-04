use std::{
    error, process,
    sync::{Arc, Mutex},
};

use colored::Colorize;
use tokio::{sync::Semaphore, task::JoinSet};

mod browser;
mod cli;
mod config;
mod display;
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
    let conf = Config::new()?;
    let mut state = State::new(Arc::clone(&conf.root))?;
    println!("Crawling");
    PERMITS.add_permits(conf.args.thread as usize);
    while state.pop_layer().is_some() {
        println!("=== Depth {} ===", state.current_depth);

        let mut handles = browse_layer(&mut state, &conf).await?;
        let childs = parse_layer(&mut state, &conf, &mut handles).await?;
        state.add_to_next_layer(childs);
        if state.current_depth == conf.args.depth {
            break;
        }
        state.current_depth += 1;
        println!();
    }

    println!("Formatting");
    format(&conf)?;

    println!("Displaying");
    display(&conf)?;
    Ok(())
}

fn format(conf: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut format = |node: &mut Node| {
        Node::format(node, &conf.args.content, &conf.args.cmd).map_err(Into::into)
    }; // Need to convert FormatError to Box< ...
    Node::explore(&conf.root, &mut format)
}

fn display(conf: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut display = |node: &mut Node| Node::display(node, &conf.args.cmd).map_err(Into::into);
    Node::explore(&conf.root, &mut display)
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
// TODO: refactor that shit
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
        let links = browser?.parse_document(&config.args.content, &parent).await;

        let links = links.into_iter().filter_map(|link| {
            if config.same_domain(&link) {
                Some(link)
            } else if state.current_external < config.args.external {
                if !explore_external {
                    explore_external = true;
                    state.current_external += 1;
                }
                Some(link)
            } else {
                None
            }
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
            eprintln!("error {:?}", e);
            process::exit(1);
        }
        return;
    }
    eprintln!("Error: can't start the tokio runtime");
    process::exit(2);
}
