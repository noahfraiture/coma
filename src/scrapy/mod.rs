pub mod crawl;
mod extract;

use colored::Colorize;
use std::fmt;
use std::sync::Arc;

pub struct Browser {
    #[allow(dead_code)] // need to keep the browser alive
    browser: headless_chrome::Browser,
    pub tab: Arc<headless_chrome::Tab>,
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
