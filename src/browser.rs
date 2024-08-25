use colored::Colorize;
use std::fmt;
use std::sync::Arc;

use anyhow::Result;
use headless_chrome::LaunchOptions;
use std::{collections::HashSet, sync::Mutex};

use scraper::Html;
use url::Url;

use crate::cli::Content;
use crate::extract;
use crate::node;

pub struct Browser {
    #[allow(dead_code)] // need to keep the browser alive
    browser: headless_chrome::Browser,
    pub tab: Arc<headless_chrome::Tab>,
}

impl Browser {
    // These functions are used in async context
    // The separation of function is needed to send connection in async
    // task, but the Html can't be sent accros async task
    pub fn new_navigate(url: &Url) -> Result<Self, BrowseError> {
        let browser = headless_chrome::Browser::new(
            LaunchOptions::default_builder()
                .devtools(false)
                .build()
                .map_err(|e| BrowseError::Browser(e.to_string()))?,
        )?;
        let tab = browser.new_tab()?;
        tab.navigate_to(url.as_str())?;
        tab.wait_until_navigated()?;
        Ok(Self { browser, tab })
    }

    // TODO : replace handle_... by the command and format
    // Add format
    // Extract useful information
    pub async fn parse_document(
        self,
        contents: &Vec<Content>,
        node: &Arc<Mutex<node::Node>>,
    ) -> HashSet<Url> {
        let response = self.tab.get_content().unwrap();
        let document = Html::parse_document(&response);
        let links = extract::extract_links(&node.lock().unwrap().url, &document);

        for content in contents {
            match content {
                Content::Texts => {
                    extract::extract_texts(node, &document);
                }
                Content::Comments => {
                    extract::extract_comments(node, &document);
                }
                Content::Links => {
                    // NOTE : here we must clone even if normally we shouldn't have
                    // twice the same content
                    node.lock().unwrap().links = Some(links.clone().into_iter().collect());
                }
                Content::Images => {
                    extract::extract_images(node, &document);
                }
                Content::Inputs => {
                    extract::extract_input(node, &document);
                }
                Content::All => {
                    extract::extract_texts(node, &document);
                    extract::extract_comments(node, &document);
                    extract::extract_images(node, &document);
                    extract::extract_input(node, &document);
                }
            };
        }
        links
    }
}

// NOTE: this is ok only because the browser is the only one using anyhow
pub enum BrowseError {
    Browser(String),
}

impl BrowseError {
    fn print(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrowseError::Browser(e) => write!(f, "{}: {}", "Browser error".red(), e),
        }
    }
}

impl fmt::Display for BrowseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f)
    }
}

impl fmt::Debug for BrowseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f)
    }
}

impl std::error::Error for BrowseError {}

impl From<anyhow::Error> for BrowseError {
    fn from(value: anyhow::Error) -> Self {
        BrowseError::Browser(value.to_string())
    }
}
