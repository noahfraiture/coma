use anyhow::Result;
use std::collections::HashSet;
use std::error::Error;
use std::{thread, time};

use headless_chrome::Browser;
use scraper::{node, Html, Node};
use url::Url;

use markup5ever::local_name;

#[allow(dead_code)]
fn extract_comments(page: &Html) -> Vec<node::Comment> {
    page.tree
        .values()
        .filter_map(|v| match v {
            Node::Comment(comment) => Some(comment.to_owned()),
            _ => None,
        })
        .collect()
}

fn extract_links(url: &Url, page: &Html) -> HashSet<Url> {
    HashSet::from_iter(page.tree.values().filter_map(|v| match v {
        Node::Element(element) => {
            let element = element.to_owned();

            // Ensure this is a link
            if !matches!(element.name.local, local_name!("a")) {
                return None;
            }

            // We want the attribute "href"
            for (key, value) in &element.attrs {
                if matches!(key.local, local_name!("href")) {
                    // TODO: add errors
                    return Url::parse(value).or_else(|_| Url::join(url, value)).ok();
                }
            }

            // I don't think this should ever happen
            None
        }
        _ => None,
    }))
}

fn extract_text(page: &Html) -> Vec<node::Text> {
    page.tree
        .values()
        .filter_map(|v| match v {
            Node::Text(text) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

fn browse_wikipedia() -> Result<(), Box<dyn Error>> {
    let browser = Browser::default()?;

    let current_tab = browser.new_tab()?;

    let url_str = "https://www.noahcode.dev";
    let url = Url::parse(url_str)?;
    println!("{:#?}", url);

    // Navigate to wikipedia
    let response = current_tab.navigate_to(url_str)?;
    thread::sleep(time::Duration::from_secs(3));

    let html = response.get_content()?;
    let document = Html::parse_document(&html);

    let comments = extract_comments(&document);
    for comment in comments {
        println!("{:#?}", comment);
    }

    let links = extract_links(&url, &document);
    for link in links {
        println!("{:#?}", link.as_str());
    }

    let texts = extract_text(&document);
    for text in texts {
        println!("{:#?}", text);
    }
    Ok(())
}

fn main() {
    let _ = browse_wikipedia();
}
