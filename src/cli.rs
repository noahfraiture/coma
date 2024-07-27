use clap::{Parser, Subcommand};

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
}

// TODO : add topology
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Extract the text in the html
    Texts,

    /// Extract the comments in the html
    Comments,

    /// Extract the links found on the page
    Links,
}

pub fn args() -> Args {
    Args::parse()
}
