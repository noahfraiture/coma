#![allow(dead_code)]
use anyhow::{Error, Result};
use std::thread;
use std::{collections::HashSet, time};

use headless_chrome::Browser;
use headless_chrome::Tab;
use markup5ever::local_name;
use scraper::{node, Html, Node};
use url::Url;

pub fn extract_comments(page: &Html) -> Vec<node::Comment> {
    page.tree
        .values()
        .filter_map(|v| match v {
            Node::Comment(comment) => Some(comment.to_owned()),
            _ => None,
        })
        .collect()
}

pub fn extract_links(url: &Url, page: &Html) -> HashSet<Url> {
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
                    // If the url is absolute, the value will replace the base url
                    return Url::join(url, value).ok();
                }
            }

            // I don't think this should ever happen
            unreachable!()
        }
        _ => None,
    }))
}

pub fn extract_texts(page: &Html) -> Vec<node::Text> {
    page.tree
        .values()
        .filter_map(|v| match v {
            Node::Text(text) => Some(text.to_owned()),
            _ => None,
        })
        .collect()
}

pub fn get_document(tab: &Tab, url: &Url) -> Result<Html, Error> {
    let response = tab.navigate_to(url.as_str())?; // FIXME
    thread::sleep(time::Duration::from_secs(3));
    let html = response.get_content()?;
    Ok(Html::parse_document(&html))
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::{thread, time};

    #[test]
    fn browse_arc() {
        let url_str = "https://arc.net/downloaded";
        let url = url::Url::parse(url_str).unwrap();
        let browser = Browser::default().unwrap();
        let tab = browser.new_tab().unwrap();

        let response = tab.navigate_to(url_str).unwrap();
        thread::sleep(time::Duration::from_secs(3));

        let html = response.get_content().unwrap();
        let document = scraper::Html::parse_document(&html);

        let links = extract_links(&url, &document);
        for link in links {
            println!("{:#?}", link.as_str());
        }

        let comments = extract_comments(&document);
        for comment in comments {
            println!("{:#?}", comment);
        }

        let texts = extract_texts(&document);
        for text in texts {
            println!("{:#?}", text);
        }
    }
}
