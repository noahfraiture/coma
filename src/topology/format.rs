use futures::future::join_all;
use std::fs::File;
use std::io::{copy, Cursor};
use tokio::task;
use url::Url;

use super::node::Node;

async fn download_img(url: &Url) -> Result<()> {
    println!("Download image");
    let (_, file_name) = url.path().rsplit_once('/').ok_or(FormatError)?;
    let response = reqwest::get(url.as_str()).await?;
    let mut file = File::create(file_name)?;

    // The file is a jpg or else
    let mut content = Cursor::new(response.bytes().await?);
    copy(&mut content, &mut file)?;
    Ok(())
}

impl Node {
    pub async fn format_images(&self) {
        let mut tasks = Vec::new();
        for image in self.images.iter().map(|v| v.to_owned()) {
            let task = task::spawn(async move {
                // TODO return error
                download_img(&image).await.unwrap();
            });
            tasks.push(task);
        }
        join_all(tasks).await;
    }

    pub fn format_comments(&self) {
        for comment in self.comments.iter() {
            println!("{}", comment);
        }
    }

    pub fn format_texts(&self) {
        for text in self.texts.iter() {
            println!("{}", text);
        }
    }

    pub fn format_inputs(&self) {
        for input in self.inputs.iter() {
            println!("{}", input);
        }
    }
}
use std::error;
use std::fmt;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
struct FormatError;

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error in format data")
    }
}

impl error::Error for FormatError {}
