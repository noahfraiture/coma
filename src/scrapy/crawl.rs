use anyhow::Result;
use headless_chrome::LaunchOptions;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use scraper::Html;
use url::Url;

use super::{extract, Browser, ScrapyError};
use crate::cli::Content;
use crate::topology;

impl Browser {
    // These functions are used in async context
    // The separation of function is needed to send connection in async
    // task, but the Html can't be sent accros async task
    pub fn new_navigate(url: &Url) -> Result<Self, ScrapyError> {
        let browser = headless_chrome::Browser::new(
            LaunchOptions::default_builder()
                .devtools(false)
                .build()
                .map_err(|e| ScrapyError::Browser(e.to_string()))?,
        )?;
        let tab = browser.new_tab()?;
        tab.navigate_to(url.as_str())?;
        tab.wait_until_navigated()?;
        Ok(Self { browser, tab })
    }

    // TODO : replace handle_... by the command and format
    // Add format
    pub async fn parse_document(
        self,
        contents: &Vec<Content>,
        parent: &Arc<Mutex<topology::Node>>,
    ) -> HashSet<Url> {
        let response = self.tab.get_content().unwrap();
        let document = Html::parse_document(&response);
        let links = extract::extract_links(&parent.lock().unwrap().url, &document);

        for content in contents {
            match content {
                Content::Texts => {
                    extract::extract_texts(parent, &document);
                }
                Content::Comments => {
                    extract::extract_comments(parent, &document);
                }
                Content::Links => {}
                Content::Images => {
                    extract::extract_images(parent, &document);
                }
                Content::Input => {
                    extract::extract_input(parent, &document);
                }
                Content::All => {
                    extract::extract_texts(parent, &document);
                    extract::extract_comments(parent, &document);
                    extract::extract_images(parent, &document);
                    extract::extract_input(parent, &document);
                }
            };
        }
        links
    }
}
