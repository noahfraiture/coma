use std::fmt;
use std::sync::Arc;

use crate::cli;
use crate::topology::Node;
use colored::Colorize;
use url::Url;

// TODO : rewrite custom functions
#[derive(PartialEq, Eq, Hash)]
pub struct Config {
    pub cmd: cli::Commands,
    pub domain: String,
    pub bound: String,
    pub target_depth: i32,
    pub thread: u32,
    pub root: Arc<Node>,
    pub graph: bool,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let args = cli::args()?;

        // NOTE: browser must still exist or the connection is closed. Pretty weird to not have
        let origin_url = Url::parse(&args.url).map_err(|e| ConfigError::Message(e.to_string()))?;
        origin_url
            .domain()
            .ok_or("Url doesn't have a domain")
            .map_err(|e| ConfigError::Message(e.to_string()))?;
        let domain = origin_url.domain().unwrap().to_owned();
        let id = origin_url.clone().to_string();

        Ok(Config {
            cmd: args.cmd,
            domain,
            bound: args.bound,
            target_depth: args.depth,
            thread: args.task,
            root: Node::new_arc(None, origin_url, id),
            graph: args.graph,
        })
    }

    pub fn same_domain(&self, url: &Url) -> bool {
        url.domain().unwrap() == self.domain
    }

    pub fn in_bound(&self, url: &Url) -> bool {
        url.as_str().contains(&self.bound)
    }
}

pub enum ConfigError {
    Message(String),
}

impl ConfigError {
    fn print(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Message(s) => {
                write!(f, "{}: {}", "Config error".red(), s)
            }
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f)
    }
}

impl fmt::Debug for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f)
    }
}

impl std::error::Error for ConfigError {}
