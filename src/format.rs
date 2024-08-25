use std::fs::File;
use std::io::{copy, Cursor};
use url::Url;

use crate::cli::{Content, Format};

use super::node::Node;

async fn download_img(url: &Url) -> Result<(), Box<dyn std::error::Error>> {
    println!("Download image");
    let (_, file_name) = url.path().rsplit_once('/').unwrap();
    let response = reqwest::get(url.as_str()).await?;
    let mut file = File::create(file_name)?;

    // The file is a jpg or else
    let mut content = Cursor::new(response.bytes().await?);
    copy(&mut content, &mut file)?;
    Ok(())
}

impl Node {
    // NOTE: we should never have an error since that means we haven't extract wanted data
    fn format(
        &mut self,
        contents: &Vec<Content>,
        f: &Format,
    ) -> std::result::Result<(), FormatError> {
        match f {
            Format::Json => {
                let mut datas: Vec<Data> = Vec::new();
                for content in contents {
                    datas.append(&mut self.format_json(content).ok_or(FormatError::None)?);
                }
                self.output = serde_json::to_string(&datas).ok();
            }
            Format::Raw => {
                let mut datas: Vec<String> = Vec::new();
                for content in contents {
                    datas.append(&mut self.format_raw(content).ok_or(FormatError::None)?)
                }
                self.output = Some(datas.join("\n"));
            }
        }
        Ok(())
    }

    fn format_raw(&mut self, content: &Content) -> Option<Vec<String>> {
        match content {
            Content::Texts => self.texts.take(),
            Content::Comments => self.comments.take(),
            Content::Links => Some(urls_string(self.links.take()?)),
            Content::Images => Some(urls_string(self.images.take()?)),
            Content::Inputs => self.inputs.take(),
            Content::All => todo!(),
        }
    }

    fn format_json(&mut self, content: &Content) -> Option<Vec<Data>> {
        match content {
            Content::Texts => Some(Data::json(self.texts.take()?, Content::Texts)),
            Content::Comments => Some(Data::json(self.comments.take()?, Content::Comments)),
            Content::Links => Some(Data::json(urls_string(self.links.take()?), Content::Links)),
            Content::Images => Some(Data::json(
                urls_string(self.images.take()?),
                Content::Images,
            )),
            Content::Inputs => Some(Data::json(self.inputs.take()?, Content::Inputs)),
            Content::All => todo!(),
        }
    }
}

fn urls_string(urls: Vec<Url>) -> Vec<String> {
    urls.into_iter().map(|link| link.to_string()).collect()
}

#[derive(serde::Serialize)]
struct Data {
    r#type: Content,
    content: String,
}

impl Data {
    fn json(datas: Vec<String>, content: Content) -> Vec<Data> {
        datas
            .into_iter()
            .map(|data| Data {
                r#type: content,
                content: data,
            })
            .collect()
    }
}

use std::error;
use std::fmt;

#[derive(Debug)]
enum FormatError {
    Serde(String),
    None,
}

impl From<serde_json::Error> for FormatError {
    fn from(value: serde_json::Error) -> Self {
        FormatError::Serde(value.to_string())
    }
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error in format data")
    }
}

impl error::Error for FormatError {}
