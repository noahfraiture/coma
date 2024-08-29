use url::Url;

use crate::cli::{Content, Display, Format};

use super::node::Node;

impl Node {
    pub fn format(
        node: &mut Node,
        contents: &Vec<Content>,
        cmd: &Display,
    ) -> std::result::Result<(), FormatError> {
        let format = match cmd {
            Display::Print { format } | Display::Save { format, .. } => format,
            _ => return Err(FormatError::Graph),
        };

        match format {
            Format::Json => Node::aggregate_json(node, contents),
            Format::Raw => Node::aggregate_raw(node, contents),
        }
    }

    fn aggregate_json(
        node: &mut Node,
        contents: &Vec<Content>,
    ) -> std::result::Result<(), FormatError> {
        let mut datas: Vec<Data> = Vec::new();
        for content in contents {
            datas.append(&mut Self::format_json(node, content));
        }
        node.output = serde_json::to_string(&datas).ok();
        Ok(())
    }

    fn aggregate_raw(
        node: &mut Node,
        contents: &Vec<Content>,
    ) -> std::result::Result<(), FormatError> {
        let mut datas: Vec<String> = Vec::new();
        for content in contents {
            datas.append(&mut Self::format_raw(node, content))
        }
        node.output = Some(datas.join("\n"));
        Ok(())
    }

    fn format_raw(node: &mut Node, content: &Content) -> Vec<String> {
        match content {
            Content::Texts => node.texts.take().unwrap(),
            Content::Comments => node.comments.take().unwrap(),
            Content::Links => urls_string(node.links.take().unwrap()),
            Content::Images => urls_string(node.images.take().unwrap()),
            Content::Inputs => node.inputs.take().unwrap(),
            Content::All => vec![
                node.texts.take().unwrap(),
                node.comments.take().unwrap(),
                urls_string(node.links.take().unwrap()),
                urls_string(node.images.take().unwrap()),
            ]
            .into_iter()
            .flatten()
            .collect(),
        }
    }

    fn format_json(node: &mut Node, content: &Content) -> Vec<Data> {
        match content {
            Content::Texts => Data::json(node.texts.take().unwrap(), Content::Texts),
            Content::Comments => Data::json(node.comments.take().unwrap(), Content::Comments),
            Content::Links => Data::json(urls_string(node.links.take().unwrap()), Content::Links),
            Content::Images => {
                Data::json(urls_string(node.images.take().unwrap()), Content::Images)
            }
            Content::Inputs => Data::json(node.inputs.take().unwrap(), Content::Inputs),
            Content::All => vec![
                Data::json(node.texts.take().unwrap(), Content::Texts),
                Data::json(node.comments.take().unwrap(), Content::Comments),
                Data::json(urls_string(node.links.take().unwrap()), Content::Links),
                Data::json(urls_string(node.images.take().unwrap()), Content::Images),
                Data::json(node.inputs.take().unwrap(), Content::Inputs),
            ]
            .into_iter()
            .flatten()
            .collect(),
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
pub enum FormatError {
    Serde,
    Graph,
}

impl From<serde_json::Error> for FormatError {
    fn from(_: serde_json::Error) -> Self {
        FormatError::Serde
    }
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error in format data")
    }
}

impl error::Error for FormatError {}
