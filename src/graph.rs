use askama::Template;
use colored::Colorize;
use serde::Serialize;
use std::{
    collections::HashSet,
    fmt, fs,
    sync::{Arc, Mutex},
};

use crate::node::Node;

#[derive(Template)]
#[template(path = "index.html")]
struct GraphTemplate {
    graph: Graph,
}

#[derive(Serialize, Debug, Clone)]
struct Graph {
    nodes: HashSet<GraphNode>,
    edges: HashSet<GraphEdge>,
}

// TODO: If i add more information, it could be great to implement
// (PartialEq, Eq, Hash) to have accurate hashset
#[derive(Serialize, Debug, Clone, Eq, PartialEq, Hash)]
struct GraphNode {
    id: String,
    label: String,
    images: Vec<String>,
    comments: Vec<String>,
    inputs: Vec<String>,
}

impl GraphNode {
    fn from_node(node: &Node) -> Self {
        Self {
            id: node.id.clone(),
            label: node.url.to_string(),
            images: node
                .images
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|url| url.to_string())
                .collect(),
            comments: node.comments.as_deref().unwrap_or_default().to_vec(),
            inputs: node.inputs.as_deref().unwrap_or_default().to_vec(),
        }
    }
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq, Hash)]
struct GraphEdge {
    from: String,
    to: String,
}

impl Graph {
    fn from_root(node: &Node) -> Self {
        let (mut nodes, mut edges) = (HashSet::<GraphNode>::new(), HashSet::<GraphEdge>::new());
        let graph_child = Graph::by_children(node);
        nodes.extend(graph_child.nodes);
        edges.extend(graph_child.edges);
        Graph { nodes, edges }
    }

    fn by_children(node: &Node) -> Self {
        let (mut nodes, mut edges) = (
            HashSet::from([GraphNode::from_node(node)]),
            HashSet::<GraphEdge>::new(),
        );
        for child in node.children.clone() {
            if !child.lock().unwrap().explored {
                continue;
            }
            edges.insert(GraphEdge {
                from: node.id.clone(),
                to: child.lock().unwrap().id.clone(),
            });
            let graph = Graph::by_children(&child.lock().unwrap());
            nodes.extend(graph.nodes);
            edges.extend(graph.edges);
        }
        Graph { nodes, edges }
    }
}

// NOTE : we suppose we search from the root and thus we will never need to look at parents
// If we want to support multiple root, we'll have to rethink this
pub async fn render(root: &Arc<Mutex<Node>>) -> Result<(), GraphError> {
    let template = GraphTemplate {
        graph: Graph::from_root(&root.lock().unwrap()),
    };
    let html = template.render().map_err(|e| GraphError(e.to_string()))?;
    let mut temp_file_path = std::env::temp_dir();
    temp_file_path.push(root.lock().unwrap().url.domain().unwrap().to_owned() + ".html");
    fs::write(&temp_file_path, html).expect("Failed to write to named file");
    let temp_file_path_str = temp_file_path.to_str().expect("Failed to get file path");
    webbrowser::open(temp_file_path_str).expect("Failed to open in web browser");
    Ok(())
}

pub struct GraphError(String);

impl GraphError {
    fn print(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", "Graph error".red(), self.0)
    }
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f)
    }
}

impl fmt::Debug for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f)
    }
}

impl std::error::Error for GraphError {}
