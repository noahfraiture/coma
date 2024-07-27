use std::{
    cell::RefCell,
    collections::{HashSet, LinkedList},
};

use anyhow::Error;
use scraper::Html;
use url::Url;

use crate::{
    cli::{self, Commands},
    scrapy::Browser,
};

pub struct State {
    browser: Browser,
    pub cmd: Commands,
    domain: String,
    depth: i32,
    visited: HashSet<String>,
    layers: LinkedList<RefCell<Vec<Url>>>,
    current_layer: RefCell<Vec<Url>>,
}

impl State {
    pub fn new() -> Self {
        let args = cli::args();

        let cmd = args.cmd;

        // NOTE: browser must still exist or the connection is closed. Pretty weird to not have
        let browser = Browser::new().expect("Error to get tab and browser");
        let origin_url = Url::parse(&args.url).expect("Url provided is not valid");
        let domain = origin_url.domain().unwrap().to_owned();

        let depth = args.depth;
        let visited: HashSet<String> = HashSet::new();

        // Queue of vector of the discovered link at the current depth
        // Each node of the linkedlist is a depth
        let mut layers: LinkedList<RefCell<Vec<Url>>> = LinkedList::new();
        layers.push_back(RefCell::new(vec![origin_url]));

        // This will never be used and could be None
        let current_layer = RefCell::new(vec![]);

        State {
            browser,
            cmd,
            domain,
            depth,
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
        self.current_layer.borrow_mut().pop()
    }

    pub fn add_next_layer(&mut self, links: HashSet<Url>) {
        if let Some(next_depth) = self.layers.front() {
            next_depth.borrow_mut().extend(links);
        } else {
            self.layers
                .push_back(RefCell::new(links.into_iter().collect()));
        }
    }

    pub fn same_domain(&self, url: &Url) -> Option<bool> {
        Some(url.domain()? == self.domain)
    }

    pub fn known(&mut self, url: &Url) -> bool {
        !self.visited.insert(url.as_str().to_string())
    }

    pub fn get_document(&self, url: &Url) -> Result<Html, Error> {
        self.browser.get_document(url)
    }

    pub fn bottom(&self) -> bool {
        self.depth == 0
    }

    pub fn deeper(&mut self) {
        self.depth -= 1;
    }
}
