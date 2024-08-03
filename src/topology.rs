use std::sync::{Arc, Mutex, Weak};
use url::Url;

pub struct Node {
    pub id: String,
    pub url: Url,
    // more infos
    pub children: Mutex<Vec<Arc<Node>>>, // Mutex is need to borrow mutability
    pub parents: Mutex<Vec<Weak<Node>>>, // Mutex is need to borrow mutability
}

impl std::hash::Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.url.hash(state);
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        other.id == self.id && other.url == self.url
    }
}

impl Eq for Node {}

impl Node {
    pub fn new_arc(parent: Option<&Arc<Node>>, url: Url, id: String) -> Arc<Node> {
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
    pub fn add_child(&self, child: &Arc<Node>) {
        self.children.lock().unwrap().push(Arc::clone(child))
    }
}
