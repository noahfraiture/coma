use crate::cli::{Display, Format};
use crate::graph;
use crate::node::Node;

impl Node {
    pub fn display(node: &mut Node, cmd: &Display) -> std::result::Result<(), CommandError> {
        match cmd {
            Display::Print { format: _ } => {
                println!("{}", node.output.as_ref().unwrap())
            }
            Display::Save { format, name } => {
                let output = node.output.as_ref().unwrap();
                let extension = match format {
                    Format::Json => "json",
                    Format::Raw => "txt",
                };

                let file_name = format!("{name}.{extension}");
                let path = Path::new(&file_name);

                let mut file = File::create(path)?;
                file.write_all(output.as_bytes())?;
            }
            Display::Graph => graph::render(node)?,
        }
        Ok(())
    }
}

use std::error;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug)]
pub enum CommandError {
    Graph,
    IO(std::io::Error),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error in command data")
    }
}

impl From<graph::GraphError> for CommandError {
    fn from(_: graph::GraphError) -> Self {
        CommandError::Graph
    }
}

impl From<std::io::Error> for CommandError {
    fn from(value: std::io::Error) -> Self {
        CommandError::IO(value)
    }
}

impl error::Error for CommandError {}
