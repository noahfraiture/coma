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
pub struct Cli {
    /// Action to perform with the data
    #[command(subcommand)]
    pub cmd: Display,

    /// Content to scrap
    #[arg(short, long, value_delimiter = ',', default_value = "all")]
    pub content: Vec<Content>,

    /// Url to start the search
    #[arg(short, long)]
    pub url: String,

    /// Depth to search from the given url, 0 for only the current url, < 0 for infinite depth
    #[arg(short, long, default_value_t = 0, allow_negative_numbers = true)]
    pub depth: i32,

    /// Upper bound in the url, any url that doesn't contains this string will be ignored
    // TODO : change default to Option<String>
    #[arg(short, long, default_value = "")]
    pub bound: String,

    /// Max number of concurrent thread
    #[arg(short, long, default_value_t = 5)]
    pub thread: u32,

    // Depth to external website with different domain. Depth have priority to stop the search
    #[arg(short, long, default_value_t = 0)]
    pub external: i32,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Display {
    /// Print the extracted content in the terminal
    Print {
        /// Format of the output
        #[arg()]
        format: Format,
    },

    /// Save the extracted content in files
    Save {
        /// Format of the output
        #[arg()]
        format: Format,

        /// Name of the output file
        #[arg(short, long, default_value = "output")]
        name: String,
    },

    /// Create a html topolgy
    Graph,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum Content {
    /// Extract the text in the html
    Texts,

    /// Extract the comments in the html
    Comments,

    /// Extract the links found on the page
    Links,

    /// Extract the images of the page
    Images,

    /// Extract informations about any form
    Inputs,

    /// Extract all information and generate a topology
    All,
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Format {
    /// Create a json file with the data
    Json,

    /// Raw data. Link are not raw href but joined with domain
    Raw,
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

pub fn args() -> Result<Cli, ArgsError> {
    let args = Cli::parse();

    match Url::parse(&args.url) {
        Ok(v) => {
            v.domain().ok_or(ArgsError::InvalidUrl(v.to_string()))?;
            Ok(args)
        }
        Err(e) => Err(ArgsError::InvalidUrl(e.to_string())),
    }
}
