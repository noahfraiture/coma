use askama::Template;

use colored::Colorize;

use crate::topology::Node;
use serde::Serialize;
use std::{
    collections::HashSet,
    fmt, fs,
    sync::{Arc, Mutex},
};

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
}

impl GraphNode {
    fn from_node(node: &Node) -> Self {
        Self {
            id: node.id.clone(),
            label: node.url.to_string(),
            images: node.images.iter().map(|url| url.to_string()).collect(),
            comments: node.comments.iter().map(|com| com.to_string()).collect(),
        }
    }
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq, Hash)]
struct GraphEdge {
    source: String,
    target: String,
}

impl Graph {
    fn from_root(node: &Node) -> Self {
        let (mut nodes, mut edges) = (HashSet::<GraphNode>::new(), HashSet::<GraphEdge>::new());
        let graph_child = Graph::by_children(node);
        let graph_parent = Graph::by_parents(node);
        nodes.extend(graph_child.nodes);
        nodes.extend(graph_parent.nodes);
        edges.extend(graph_child.edges);
        edges.extend(graph_parent.edges);
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
                source: node.id.clone(),
                target: child.lock().unwrap().id.clone(),
            });
            let graph = Graph::by_children(&child.lock().unwrap());
            nodes.extend(graph.nodes);
            edges.extend(graph.edges);
        }
        Graph { nodes, edges }
    }

    fn by_parents(node: &Node) -> Self {
        let (mut nodes, mut edges) = (
            HashSet::from([GraphNode::from_node(node)]),
            HashSet::<GraphEdge>::new(),
        );
        for parent in node.parents.clone() {
            if let Some(parent) = parent.upgrade() {
                if !parent.lock().unwrap().explored {
                    unreachable!("Parent has not been explored") // debug purpose
                }
                edges.insert(GraphEdge {
                    source: node.id.clone(),
                    target: parent.lock().unwrap().id.clone(),
                });
                let graph = Graph::by_parents(&parent.lock().unwrap());
                nodes.extend(graph.nodes);
                edges.extend(graph.edges);
            }
        }
        Graph { nodes, edges }
    }
}

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
