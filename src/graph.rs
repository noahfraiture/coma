use askama::Template;

use colored::Colorize;
use webbrowser;

use crate::topology::Node;
use serde::Serialize;
use std::{collections::HashSet, fmt, fs, sync::Arc, thread, time::Duration};

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
}

impl GraphNode {
    fn from_node(node: &Node) -> Self {
        Self {
            id: node.id.clone(),
            label: node.url.to_string(),
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
        for child in node.children.lock().unwrap().clone() {
            edges.insert(GraphEdge {
                source: node.id.clone(),
                target: child.id.clone(),
            });
            let graph = Graph::by_children(&child);
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
        for parent in node.parents.lock().unwrap().clone() {
            if let Some(parent) = parent.upgrade() {
                edges.insert(GraphEdge {
                    source: node.id.clone(),
                    target: parent.id.clone(),
                });
                let graph = Graph::by_parents(&parent);
                nodes.extend(graph.nodes);
                edges.extend(graph.edges);
            }
        }
        Graph { nodes, edges }
    }
}

pub async fn render(root: &Arc<Node>) -> Result<(), GraphError> {
    let template = GraphTemplate {
        graph: Graph::from_root(root),
    };
    let html = template.render().map_err(|e| GraphError(e.to_string()))?;
    let mut temp_file_path = std::env::temp_dir();
    temp_file_path.push(root.url.domain().unwrap().to_owned() + ".html");
    fs::write(&temp_file_path, html).expect("Failed to write to named file");
    let temp_file_path_str = temp_file_path.to_str().expect("Failed to get file path");
    webbrowser::open(temp_file_path_str).expect("Failed to open in web browser");

    // Give time for the browser to open and load the page
    thread::sleep(Duration::from_secs(3));
    fs::remove_file(temp_file_path).expect("Failed to delete the file");
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, sync::Arc, thread, time::Duration};
    use url::Url;

    fn node_example() -> Arc<Node> {
        // pretty draw in static
        let url = Url::parse("http://google.com").unwrap();
        let t0 = Node::new_arc(None, url.clone(), String::from("0"));

        let t01 = Node::new_arc(Some(&t0), url.clone(), String::from("1"));
        t0.add_child(&t01);
        let t02 = Node::new_arc(Some(&t0), url.clone(), String::from("2"));
        t0.add_child(&t02);

        let t011 = Node::new_arc(Some(&t01), url.clone(), String::from("3"));
        t01.add_child(&t011);
        t0.add_child(&t011);

        let t012 = Node::new_arc(Some(&t01), url.clone(), String::from("4"));
        t01.add_child(&t012);

        let t013 = Node::new_arc(Some(&t01), url.clone(), String::from("5"));
        t01.add_child(&t013);

        let t022 = Node::new_arc(Some(&t02), url.clone(), String::from("6"));
        t02.add_child(&t022);
        t02.add_child(&t013);

        t0
    }

    #[test]
    fn generate_html() {
        let node = node_example();
        let graph = Graph::from_root(&node);
        let template = GraphTemplate { graph };
        let html = template.render().unwrap();
        let mut temp_file_path = std::env::temp_dir();
        temp_file_path.push(node.url.domain().unwrap().to_owned() + ".html");
        fs::write(&temp_file_path, html).expect("Failed to write to named file");
        let temp_file_path_str = temp_file_path.to_str().expect("Failed to get file path");
        webbrowser::open(temp_file_path_str).expect("Failed to open in web browser");

        // Give time for the browser to open and load the page
        thread::sleep(Duration::from_secs(3));
        fs::remove_file(temp_file_path).expect("Failed to delete the file");
    }
}
