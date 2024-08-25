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
        let mut datas: Vec<String> = Vec::new();
        for content in contents {
            match content {
                Content::Texts => datas.append(&mut self.format_texts(f)?),
                Content::Comments => datas.append(&mut self.format_comments(f)?),
                Content::Links => datas.append(&mut self.format_links(f)?),
                Content::Images => datas.append(&mut self.format_images(f)?),
                Content::Inputs => datas.append(&mut self.format_inputs(f)?),
                Content::All => {
                    datas.append(&mut self.format_texts(f)?);
                    datas.append(&mut self.format_comments(f)?);
                    datas.append(&mut self.format_links(f)?);
                    datas.append(&mut self.format_images(f)?);
                    datas.append(&mut self.format_inputs(f)?);
                }
            };
        }
        self.output = Some(match f {
            Format::Json => serde_json::to_string(&datas)?,
            Format::Raw => datas.join("\n"),
        });
        Ok(())
    }

    fn format_texts(&mut self, f: &Format) -> Result<Vec<String>, FormatError> {
        match f {
            Format::Json => Data::json(self.texts.take().unwrap(), Content::Texts),
            Format::Raw => Ok(self.texts.take().unwrap()),
        }
    }
    fn format_comments(&mut self, f: &Format) -> Result<Vec<String>, FormatError> {
        match f {
            Format::Json => Data::json(self.comments.take().unwrap(), Content::Comments),
            Format::Raw => Ok(self.comments.take().unwrap()),
        }
    }
    fn format_links(&mut self, f: &Format) -> Result<Vec<String>, FormatError> {
        match f {
            Format::Json => Data::json(url_to_string(self.links.take().unwrap()), Content::Links),
            Format::Raw => Ok(url_to_string(self.links.take().unwrap())),
        }
    }
    fn format_images(&mut self, f: &Format) -> Result<Vec<String>, FormatError> {
        match f {
            Format::Json => Data::json(url_to_string(self.images.take().unwrap()), Content::Images),
            Format::Raw => Ok(url_to_string(self.images.take().unwrap())),
        }
    }
    fn format_inputs(&mut self, f: &Format) -> Result<Vec<String>, FormatError> {
        match f {
            Format::Json => Data::json(self.inputs.take().unwrap(), Content::Inputs),
            Format::Raw => Ok(self.inputs.take().unwrap()),
        }
    }
}

fn url_to_string(urls: Vec<Url>) -> Vec<String> {
    urls.into_iter().map(|link| link.to_string()).collect()
}

#[derive(serde::Serialize)]
struct Data {
    r#type: Content,
    content: String,
}

impl Data {
    fn json(datas: Vec<String>, content: Content) -> Result<Vec<String>, FormatError> {
        Ok(datas
            .into_iter()
            .map(|data| Data {
                r#type: content,
                content: data,
            })
            .map(|data| serde_json::to_string(&data))
            .collect::<Result<Vec<String>, serde_json::Error>>()?)
    }
}

use std::error;
use std::fmt;

#[derive(Debug)]
struct FormatError(String);

impl From<serde_json::Error> for FormatError {
    fn from(value: serde_json::Error) -> Self {
        FormatError(value.to_string())
    }
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error in format data")
    }
}

impl error::Error for FormatError {}
