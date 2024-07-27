use std::collections::HashSet;

use cli::Commands;
use colored::Colorize;
use scraper::Html;
use url::Url;

mod cli;
mod scrapy;
mod state;

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

fn run() {
    let mut state = state::State::new();
    while state.pop_layer().is_some() {
        while let Some(url) = state.next_url() {
            // TODO: remove unwrap
            if !state.same_domain(&url).unwrap() || state.known(&url) {
                continue;
            }

            let document = state.get_document(&url).unwrap();
            println!("Visiting {}", url.as_str().green());

            let links = scrapy::extract_links(&url, &document);
            handle_result(state.cmd, &document, &links);
            state.add_next_layer(links);
        }
        if state.bottom() {
            break;
        }
        state.deeper();
    }
}

fn main() {
    run();
}
