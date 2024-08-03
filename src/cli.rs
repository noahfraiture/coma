use core::fmt;

use clap::{Parser, Subcommand};
use colored::Colorize;
use url::Url;

/// Website scraper
#[derive(Parser, Debug)]
#[command(name = "Coma")]
#[command(author = "Noah")]
#[command(version)]
#[command(help_template = "
{name} - {about}

Author: {author}
Version: {version}

{usage-heading} {usage}
{all-args} {tab}")]
pub struct Args {
    /// Informations extracted
    #[command(subcommand)]
    pub cmd: Commands,

    /// Url to start the search
    #[arg(short, long)]
    pub url: String,

    /// Depth to search from the given url, 0 for only the current url, < 0 for infinite depth
    #[arg(short, long, default_value_t = 0, allow_negative_numbers = true)]
    pub depth: i32,

    /// Upper bound in the url, any url that doesn't contains this string will be ignored
    #[arg(short, long, default_value = "")]
    pub bound: String,

    /// Max number of concurrent task
    #[arg(short, long, default_value_t = 5)]
    pub task: u32,

    /// Display a graph in the browser
    #[arg(short, long, default_value_t = false)]
    pub graph: bool,
}

// TODO : add topology
#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Commands {
    /// Extract the text in the html
    Texts,

    /// Extract the comments in the html
    Comments,

    /// Extract the links found on the page
    Links,

    /// Extract the images of the page
    Images,
}

pub enum ArgsError {
    InvalidUrl(String),
}

impl ArgsError {
    fn print(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgsError::InvalidUrl(url) => write!(f, "{}: {}", "Invalid URL".red(), url),
        }
    }
}

impl fmt::Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f)
    }
}

impl fmt::Debug for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f)
    }
}

impl std::error::Error for ArgsError {}

pub fn args() -> Result<Args, ArgsError> {
    let args = Args::parse();

    if let Err(e) = Url::parse(&args.url) {
        Err(ArgsError::InvalidUrl(e.to_string()))
    } else {
        Ok(args)
    }
}
