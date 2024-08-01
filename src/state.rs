use crate::cli::{self, Commands};
use colored::Colorize;
use std::{
    collections::{HashSet, LinkedList},
    fmt,
    sync::Mutex,
};
use url::Url;

pub struct State {
    pub cmd: Commands,
    domain: String,
    bound: String,
    pub current_depth: i32,
    target_depth: i32,
    visited: HashSet<String>,
    layers: LinkedList<Mutex<Vec<Url>>>,
    current_layer: Mutex<Vec<Url>>,
    pub thread: u32,
}

impl State {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let args = cli::args()?;

        let cmd = args.cmd;

        // NOTE: browser must still exist or the connection is closed. Pretty weird to not have
        let origin_url = Url::parse(&args.url).map_err(|e| StateError::Message(e.to_string()))?;
        origin_url
            .domain()
            .ok_or("Url doesn't have a domain")
            .map_err(|e| StateError::Message(e.to_string()))?;
        let domain = origin_url.domain().unwrap().to_owned();

        let bound = args.bound;
        let current_depth = 0;
        let target_depth = args.depth;
        let thread = args.thread;
        let visited: HashSet<String> = HashSet::new();

        // Queue of vector of the discovered link at the current depth
        // Each node of the linkedlist is a depth
        let mut layers: LinkedList<Mutex<Vec<Url>>> = LinkedList::new();
        layers.push_back(Mutex::new(vec![origin_url]));

        // This will never be used and could be None
        let current_layer = Mutex::new(vec![]);

        Ok(State {
            cmd,
            domain,
            bound,
            current_depth,
            target_depth,
            thread,
            visited,
            layers,
            current_layer,
        })
    }

    pub fn pop_layer(&mut self) -> Option<()> {
        self.current_layer = self.layers.pop_front()?;
        Some(())
    }

    pub fn next_url(&self) -> Result<Option<Url>, StateError> {
        if let Ok(mut layer) = self.current_layer.lock() {
            Ok(layer.pop())
        } else {
            Err(StateError::Lock)
        }
    }

    pub fn add_to_next_layer(
        &mut self,
        links: HashSet<Url>,
    ) -> std::result::Result<(), StateError> {
        if self.layers.front().is_none() {
            self.layers.push_back(Mutex::new(vec![]));
        }
        if let Ok(mut layer) = self.layers.front().unwrap().lock() {
            layer.extend(links);
            Ok(())
        } else {
            Err(StateError::Message(String::from("Error to lock")))
        }
    }

    pub fn same_domain(&self, url: &Url) -> bool {
        // Url has been checked already
        url.domain().unwrap() == self.domain
    }

    pub fn known(&mut self, url: &Url) -> bool {
        !self.visited.insert(url.as_str().to_string())
    }

    pub fn bottom(&self) -> bool {
        self.current_depth == self.target_depth
    }

    pub fn deeper(&mut self) {
        self.current_depth += 1;
    }

    pub fn in_bound(&self, url: &Url) -> bool {
        url.as_str().contains(&self.bound)
    }
}

pub enum StateError {
    Lock,
    Message(String),
}

impl StateError {
    fn print(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::Lock => {
                write!(f, "{}: {}", "State error".red(), "locking mutex problem")
            }
            StateError::Message(s) => {
                write!(f, "{}: {}", "State error".red(), s)
            }
        }
    }
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f)
    }
}

impl fmt::Debug for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f)
    }
}

impl std::error::Error for StateError {}
