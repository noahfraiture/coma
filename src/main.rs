use anyhow::Result;
use std::error::Error;

use headless_chrome::Browser;
use scraper::{node, Html, Node};
use url::Url;

use markup5ever::local_name;

fn extract_comments(page: &Html) -> Vec<node::Comment> {
    page.tree
        .values()
        .filter_map(|v| match v {
            Node::Comment(comment) => Some(comment.to_owned()),
            _ => None,
        })
        .collect()
}

fn extract_links(url: Url, page: &Html) -> Result<Vec<Url>, ()> {
    page.tree
        .values()
        .filter_map(|v| match v {
            Node::Element(element) => {
                // TODO: check if it's link
                if matches!(element.name.local, local_name!("a")) {
                    let link = "";
                    Some(url.join(link)?)
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect()
}

fn browse_wikipedia() -> Result<(), Box<dyn Error>> {
    let browser = Browser::default()?;

    let current_tab = browser.new_tab()?;

    // Navigate to wikipedia
    let response = current_tab.navigate_to(
        "https://docs.rs/markup5ever/0.12.0/markup5ever/interface/struct.QualName.html",
    )?;
    let html = response.get_content()?;
    let document = Html::parse_document(&html);

    let links = extract_links(&document);
    println!("{:#?}", links);
    Ok(())
}

fn main() {
    let _ = browse_wikipedia();
}
