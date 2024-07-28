use crate::cli::{self, Commands};
use std::{
    collections::{HashSet, LinkedList},
    sync::Mutex,
};
use url::Url;

pub struct State {
    pub cmd: Commands,
    domain: String,
    bound: String,
    depth: i32,
    visited: HashSet<String>,
    layers: LinkedList<Mutex<Vec<Url>>>,
    current_layer: Mutex<Vec<Url>>,
    pub thread: u32,
}

impl State {
    pub fn new() -> Self {
        let args = cli::args();

        let cmd = args.cmd;

        // NOTE: browser must still exist or the connection is closed. Pretty weird to not have
        let origin_url = Url::parse(&args.url).expect("Url provided is not valid");
        let domain = origin_url.domain().unwrap().to_owned();

        let bound = args.bound;
        let depth = args.depth;
        let thread = args.thread;
        let visited: HashSet<String> = HashSet::new();

        // Queue of vector of the discovered link at the current depth
        // Each node of the linkedlist is a depth
        let mut layers: LinkedList<Mutex<Vec<Url>>> = LinkedList::new();
        layers.push_back(Mutex::new(vec![origin_url]));

        // This will never be used and could be None
        let current_layer = Mutex::new(vec![]);

        State {
            cmd,
            domain,
            bound,
            depth,
            thread,
            visited,
            layers,
            current_layer,
        }
    }

    pub fn pop_layer(&mut self) -> Option<()> {
        self.current_layer = self.layers.pop_front()?;
        Some(())
    }

    pub fn next_url(&self) -> Option<Url> {
        (*self.current_layer.lock().unwrap()).pop()
    }

    pub fn add_to_next_layer(&mut self, links: HashSet<Url>) {
        if self.layers.front().is_none() {
            self.layers.push_back(Mutex::new(vec![]));
        }
        (*self.layers.front().unwrap().lock().unwrap()).extend(links);
    }

    pub fn same_domain(&self, url: &Url) -> Option<bool> {
        Some(url.domain()? == self.domain)
    }

    pub fn known(&mut self, url: &Url) -> bool {
        !self.visited.insert(url.as_str().to_string())
    }

    pub fn bottom(&self) -> bool {
        self.depth == 0
    }

    pub fn deeper(&mut self) {
        self.depth -= 1;
    }

    pub fn in_bound(&self, url: &Url) -> bool {
        url.as_str().contains(&self.bound)
    }
}
