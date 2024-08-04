use anyhow::{Context, Result};
use headless_chrome::LaunchOptions;
use std::sync::Arc;
use std::{
    collections::HashSet,
    fs::File,
    io::{copy, Cursor},
};

use scraper::Html;
use url::Url;

use super::{extract, Browser, ScrapyError};
use crate::cli::Commands;
use crate::topology;

pub async fn download_img(url: &Url) -> Result<()> {
    println!("Download image");
    let (_, file_name) = url.path().rsplit_once('/').context("error on split")?;
    let response = reqwest::get(url.as_str()).await?;
    let mut file = File::create(file_name)?;

    // The file is a jpg or else
    let mut content = Cursor::new(response.bytes().await?);
    copy(&mut content, &mut file)?;
    Ok(())
}

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

    pub async fn parse_document(self, cmd: Commands, parent: &Arc<topology::Node>) -> HashSet<Url> {
        let response = self.tab.get_content().unwrap();
        let document = Html::parse_document(&response);
        let links = extract::extract_links(&parent.url, &document);
        match cmd {
            Commands::Texts => {
                extract::extract_texts(parent, &document);
                parent.handle_texts();
            }
            Commands::Comments => {
                extract::extract_comments(parent, &document);
                parent.handle_comments();
            }
            Commands::Images => {
                extract::extract_images(parent, &document);
                parent.handle_images().await;
            }
            Commands::Links => {}
            Commands::Graph => {
                extract::extract_texts(parent, &document);
                extract::extract_comments(parent, &document);
                extract::extract_images(parent, &document);
            }
        };
        links
    }
}
