#![allow(dead_code)]
use anyhow::{Context, Result};
use std::fmt;
use std::sync::Arc;
use std::{
    collections::HashSet,
    fs::File,
    io::{copy, Cursor},
};

use colored::Colorize;
use markup5ever::local_name;
use scraper::{node, Html, Node};
use url::Url;

use crate::cli::Commands;

// TODO : add async
async fn download_img(url: &Url) -> Result<()> {
    let (_, file_name) = url.path().rsplit_once('/').context("error on split")?;
    let response = reqwest::get(url.as_str()).await?;
    let mut file = File::create(file_name)?;

    // The file is a jpg or else
    let mut content = Cursor::new(response.bytes().await?);
    copy(&mut content, &mut file)?;
    Ok(())
}

pub fn extract_comments(page: &Html) -> Vec<node::Comment> {
    page.tree
        .values()
        .filter_map(|v| match v {
            Node::Comment(comment) => Some(comment.to_owned()),
            _ => None,
        })
        .collect()
}

// TODO: here hashset but other vector, should harmonized that
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
            None
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

pub fn extract_images(url: &Url, page: &Html) -> Vec<Url> {
    page.tree
        .values()
        .filter_map(|v| match v {
            Node::Element(element) => {
                let element = element.to_owned();

                // Ensure this is a link
                if !matches!(element.name.local, local_name!("img")) {
                    return None;
                }

                // We want the attribute "href"
                for (key, value) in &element.attrs {
                    if matches!(key.local, local_name!("src")) {
                        // TODO: add errors
                        // If the url is absolute, the value will replace the base url
                        return Url::join(url, value).ok();
                    }
                }
                None
            }
            _ => None,
        })
        .collect()
}

pub struct Browser {
    browser: headless_chrome::Browser,
    pub tab: Arc<headless_chrome::Tab>,
}

impl Browser {
    // These functions are used in async context
    // The separation of function is needed to send connection in async
    // task, but the Html can't be sent accros async task
    pub fn new_navigate(url: &Url) -> Result<Self, ScrapyError> {
        let browser = headless_chrome::Browser::default()?;
        let tab = browser.new_tab()?;
        tab.navigate_to(url.as_str())?;
        tab.wait_until_navigated()?;
        Ok(Self { browser, tab })
    }

    pub async fn parse_document(self, cmd: Commands, url: &Url) -> (HashSet<Url>, usize) {
        // NOTE: could refactor to dellocate this
        let response = self.tab.get_content().unwrap();
        let document = Html::parse_document(&response);
        let links = extract_links(url, &document);
        let mut count = 0;
        match cmd {
            Commands::Texts => {
                let texts = extract_texts(&document);
                count += texts.len();
                for text in texts {
                    println!("{:#?}", text);
                }
            }
            Commands::Comments => {
                let comments = extract_comments(&document);
                count += comments.len();
                for comment in comments {
                    println!("{:#?}", comment);
                }
            }
            Commands::Images => {
                let images = extract_images(url, &document);
                count += images.len();
                for image in images {
                    // TODO take care of error
                    let _ = download_img(&image).await;
                }
            }
            Commands::Links => {
                count += links.len();
                for link in &links {
                    println!("{:#?}", link.as_str());
                }
            }
        };
        (links, count)
    }
}

// NOTE: this is ok only because the browser is the only one using anyhow
pub enum ScrapyError {
    Browser(String),
}

impl ScrapyError {
    fn print(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScrapyError::Browser(e) => write!(f, "{}: {}", "Browser error".red(), e),
        }
    }
}

impl fmt::Display for ScrapyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f)
    }
}

impl fmt::Debug for ScrapyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f)
    }
}

impl std::error::Error for ScrapyError {}

impl From<anyhow::Error> for ScrapyError {
    fn from(value: anyhow::Error) -> Self {
        ScrapyError::Browser(value.to_string())
    }
}
