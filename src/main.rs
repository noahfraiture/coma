use std::collections::HashSet;

use cli::Commands;
use headless_chrome::Browser;
use url::Url;

mod cli;
mod scrapy;

fn main() {
    let args = cli::args();
    let browser = Browser::default().expect("Error to instance browser");
    let tab = browser.new_tab().expect("Error in instance tab");

    let origin_url = Url::parse(&args.url).expect("Url provided is not valid");
    let domain = origin_url.domain();

    let mut urls = vec![origin_url.clone()];
    let mut visited: HashSet<String> = HashSet::new();
    while let Some(url) = urls.pop() {
        if url.domain() != domain {
            continue;
        }

        // The url as already been visited
        if !visited.insert(url.as_str().to_string()) {
            continue;
        }

        let document = scrapy::get_document(&tab, &url).expect("Error in parsing the document");
        let links = scrapy::extract_links(&url, &document);

        match args.cmd {
            Commands::Texts => {
                let texts = scrapy::extract_texts(&document);
                for text in texts {
                    println!("{:#?}", text);
                }
            }
            Commands::Comments => {
                let comments = scrapy::extract_comments(&document);
                for comment in comments {
                    println!("{:#?}", comment);
                }
            }
            Commands::Links => {
                for link in &links {
                    println!("{:#?}", link.as_str());
                }
            }
        }

        // We extend here because the urls.extends move the links
        visited.extend(links.iter().map(|link| link.as_str().to_string()));
        urls.extend(links);
    }
}
