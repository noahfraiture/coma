use std::sync::{Arc, Mutex, Weak};
use url::Url;

pub struct Node {
    pub id: String,
    pub url: Url,
    // TODO : should I move while Node behind of mutex instead of most field ?
    // TODO : I could use a color to show difference between explored and unexplored node
    pub explored: bool, // flag used to know if it will be rendered
    // Every node will own every images on the page
    // More logic that every node own a copy of the url to the image

    // Mutex is need to borrow mutability of Arc
    pub images: Option<Vec<Url>>,
    // Can't directly use scraper::node::{Comment, Text} since their aren't Send/Sync
    // Could try later to impl these trait
    pub comments: Option<Vec<String>>,
    pub texts: Option<Vec<String>>,
    pub inputs: Option<Vec<String>>,
    pub links: Option<Vec<Url>>,
    pub children: Vec<Arc<Mutex<Node>>>,
    pub parents: Vec<Weak<Mutex<Node>>>,

    // Output already formatted as wanted
    pub output: Option<String>,
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
    pub fn new_arc(parent: Option<&Arc<Mutex<Node>>>, url: Url, id: String) -> Arc<Mutex<Node>> {
        let node = Arc::new(Mutex::new(Node {
            id,
            url,
            explored: false,
            images: None,
            comments: None,
            texts: None,
            inputs: None,
            links: None,
            children: vec![],
            parents: parent.map_or_else(Vec::new, |p| vec![Arc::downgrade(p)]),
            output: None,
        }));
        if let Some(parent) = parent {
            parent.lock().unwrap().add_child(&node);
        };
        node
    }

    pub fn explore(
        node: &Arc<Mutex<Node>>,
        func: Visitor,
    ) -> Result<(), Box<dyn std::error::Error>> {
        func(&mut node.lock().unwrap())?;
        for child in &node.lock().unwrap().children {
            if !child.lock().unwrap().explored {
                continue;
            }
            Node::explore(child, func)?;
        }
        Ok(())
    }

    pub fn add_child(&mut self, child: &Arc<Mutex<Node>>) {
        self.children.push(Arc::clone(child))
    }

    pub fn quantity_elements(&self) -> usize {
        self.images.as_ref().unwrap_or(&vec![]).len()
            + self.comments.as_ref().unwrap_or(&vec![]).len()
            + self.texts.as_ref().unwrap_or(&vec![]).len()
    }
}

type Visitor<'a> = &'a mut dyn FnMut(&mut Node) -> Result<(), Box<dyn std::error::Error>>;
