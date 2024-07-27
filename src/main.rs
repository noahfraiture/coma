use std::{
    cell::RefCell,
    collections::{HashSet, LinkedList},
};

use cli::Commands;
use colored::Colorize;
use scrapy::Browser;
use url::Url;

mod cli;
mod scrapy;

fn main() {
    let args = cli::args();

    // NOTE: browser must still exist or the connection is closed. Pretty weird to not have
    let browser = Browser::new().expect("Error to get tab and browser");
    let origin_url = Url::parse(&args.url).expect("Url provided is not valid");
    let domain = origin_url.domain();

    let mut depth = args.depth;

    let mut visited: HashSet<String> = HashSet::new();

    // Queue of vector of the discovered link at the current depth
    // Each node of the linkedlist is a depth
    let mut urls_list: LinkedList<RefCell<Vec<Url>>> = LinkedList::new();
    urls_list.push_back(RefCell::new(vec![origin_url.clone()]));

    while let Some(urls) = urls_list.pop_front() {
        while let Some(url) = urls.borrow_mut().pop() {
            if url.domain() != domain {
                continue;
            }

            // The url as already been visited
            if !visited.insert(url.as_str().to_string()) {
                continue;
            }

            let document = browser
                .get_document(&url)
                .expect("Error in parsing the document");
            println!("Visiting {}", url.as_str().green());
            let links = scrapy::extract_links(&url, &document);

            // NOTE: could refactor to dellocate this
            match args.cmd {
                Commands::Texts => {
                    let texts = scrapy::extract_texts(&document);
                    println!("Found {} texts", texts.len().to_string().green());
                    for text in texts {
                        println!("{:#?}", text);
                    }
                }
                Commands::Comments => {
                    let comments = scrapy::extract_comments(&document);
                    println!("Found {} comments", comments.len().to_string().green());
                    for comment in comments {
                        println!("{:#?}", comment);
                    }
                }
                Commands::Links => {
                    println!("Found {} links", links.len().to_string().green());
                    for link in &links {
                        println!("{:#?}", link.as_str());
                    }
                }
            }

            // We extend here because the urls.extends move the links
            if let Some(next_depth) = urls_list.front() {
                next_depth.borrow_mut().extend(links);
            } else {
                urls_list.push_back(RefCell::new(links.into_iter().collect()));
            }
        }
        if depth == 0 {
            break;
        }
        depth -= 1;
    }
}
