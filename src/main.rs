use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::{HashSet, LinkedList},
};

use cli::{Args, Commands};
use colored::Colorize;
use scraper::Html;
use scrapy::Browser;
use url::Url;

mod cli;
mod scrapy;

fn handle_result(cmd: Commands, document: &Html, links: &HashSet<Url>) {
    // NOTE: could refactor to dellocate this
    match cmd {
        Commands::Texts => {
            let texts = scrapy::extract_texts(document);
            println!("Found {} texts", texts.len().to_string().green());
            for text in texts {
                println!("{:#?}", text);
            }
        }
        Commands::Comments => {
            let comments = scrapy::extract_comments(document);
            println!("Found {} comments", comments.len().to_string().green());
            for comment in comments {
                println!("{:#?}", comment);
            }
        }
        Commands::Links => {
            println!("Found {} links", links.len().to_string().green());
            for link in links {
                println!("{:#?}", link.as_str());
            }
        }
    }
}

struct State {
    browser: Browser,
    cmd: Commands,
    domain: String,
    depth: i32,
    visited: HashSet<String>,
    layers: LinkedList<RefCell<Vec<Url>>>,
    current_layer: RefCell<Vec<Url>>,
}

impl State {
    fn new() -> Self {
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

    fn pop_layer(&mut self) -> Option<()> {
        self.current_layer = self.layers.pop_front()?;
        Some(())
    }

    fn next_url(&self) -> Option<Url> {
        self.current_layer.borrow_mut().pop()
    }
}

fn run() {
    let mut state = State::new();
    while state.pop_layer().is_some() {
        while let Some(url) = state.next_url() {
            if url.domain().unwrap() != state.domain
                || !state.visited.insert(url.as_str().to_string())
            {
                continue;
            }

            let document = state
                .browser
                .get_document(&url)
                .expect("Error in parsing the document");
            println!("Visiting {}", url.as_str().green());

            let links = scrapy::extract_links(&url, &document);

            handle_result(state.cmd, &document, &links);

            // We extend here because the urls.extends move the links
            if let Some(next_depth) = state.layers.front() {
                next_depth.borrow_mut().extend(links);
            } else {
                state
                    .layers
                    .push_back(RefCell::new(links.into_iter().collect()));
            }
        }
        if state.depth == 0 {
            break;
        }
        state.depth -= 1;
    }
}

fn main() {
    run();
}
