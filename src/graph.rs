use actix_files::NamedFile;
use actix_web::{web, App, HttpServer, Responder};
use url::Url;

use serde::Serialize;
use std::{
    collections::{hash_set, HashSet},
    sync::{Arc, Mutex, Weak},
};

struct Node {
    id: String,
    url: Url,
    // more infos
    children: Mutex<Vec<Arc<Node>>>,
    parents: Mutex<Vec<Weak<Node>>>,
}

impl Node {
    fn new_arc(parent: Option<&Arc<Node>>, url: Url, id: String) -> Arc<Node> {
        let node = Arc::new(Node {
            url,
            id,
            children: Mutex::new(vec![]),
            parents: Mutex::new(parent.map_or_else(Vec::new, |p| vec![Arc::downgrade(p)])),
        });
        if let Some(parent) = parent {
            parent.add_child(&node);
        };
        node
    }

    // TODO : remove unwrap
    fn add_child(&self, child: &Arc<Node>) {
        self.children.lock().unwrap().push(Arc::clone(child))
    }
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

fn from_root(node: &Node) -> Graph {
    let (mut nodes, mut edges) = (HashSet::<GraphNode>::new(), HashSet::<GraphEdge>::new());
    let graph_child = by_children(node);
    let graph_parent = by_parents(node);
    nodes.extend(graph_child.nodes);
    nodes.extend(graph_parent.nodes);
    edges.extend(graph_child.edges);
    edges.extend(graph_parent.edges);
    Graph { nodes, edges }
}

fn by_children(node: &Node) -> Graph {
    let (mut nodes, mut edges) = (
        HashSet::from([GraphNode::from_node(node)]),
        HashSet::<GraphEdge>::new(),
    );
    for child in node.children.lock().unwrap().clone() {
        edges.insert(GraphEdge {
            source: node.id.clone(),
            target: child.id.clone(),
        });
        let graph = by_children(&child);
        println!("Nodes before : {:?}", nodes);
        println!("graph : {:?}", graph.nodes);
        nodes.extend(graph.nodes);
        println!("Nodes after: {:?}", nodes);
        edges.extend(graph.edges);
    }
    Graph { nodes, edges }
}

fn by_parents(node: &Node) -> Graph {
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
            let graph = by_parents(&parent);
            nodes.extend(graph.nodes);
            edges.extend(graph.edges);
        }
    }
    Graph { nodes, edges }
}

async fn index() -> impl Responder {
    NamedFile::open("static/index.html")
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn graph_example() -> impl Responder {
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

        let graph = from_root(&t0);
        let x = web::Json(graph);
        println!("{:#?}", x);
        x
    }

    #[tokio::test]
    async fn example() {
        graph_example().await;
    }

    #[tokio::test]
    async fn run() -> std::io::Result<()> {
        HttpServer::new(|| {
            App::new()
                .route("/", web::get().to(index))
                .route("/graph", web::get().to(graph_example))
        })
        .bind("127.0.0.1:8080")?
        .run()
        .await
    }
}
