use futures::future::join_all;
use std::sync::{Arc, Mutex, Weak};
use tokio::task;
use url::Url;

use crate::scrapy::crawl;

pub struct Node {
    pub id: String,
    pub url: Url,
    // TODO : should I move while Node behind of mutex instead of most field ?
    pub explored: bool, // flag used to know if it will be rendered
    // Every node will own every images on the page
    // More logic that every node own a copy of the url to the image

    // Mutex is need to borrow mutability of Arc
    pub images: Vec<Url>,
    // Can't directly use scraper::node::{Comment, Text} since their aren't Send/Sync
    // Could try later to impl these trait
    pub comments: Vec<String>,
    pub texts: Vec<String>,
    pub inputs: Vec<String>,
    pub children: Vec<Arc<Mutex<Node>>>,
    pub parents: Vec<Weak<Mutex<Node>>>,
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
            images: Vec::new(),
            comments: Vec::new(),
            texts: Vec::new(),
            inputs: Vec::new(),
            children: vec![],
            parents: parent.map_or_else(Vec::new, |p| vec![Arc::downgrade(p)]),
        }));
        if let Some(parent) = parent {
            parent.lock().unwrap().add_child(&node);
        };
        node
    }

    // TODO : remove unwrap
    pub fn add_child(&mut self, child: &Arc<Mutex<Node>>) {
        self.children.push(Arc::clone(child))
    }

    pub fn explore(&mut self) {
        self.explored = true;
    }

    pub fn quantity_elements(&self) -> usize {
        self.images.len() + self.comments.len() + self.texts.len()
    }

    pub async fn handle_images(&self) {
        let mut tasks = Vec::new();
        for image in self.images.iter().map(|v| v.to_owned()) {
            let task = task::spawn(async move {
                // TODO return error
                crawl::download_img(&image).await.unwrap();
            });
            tasks.push(task);
        }
        join_all(tasks).await;
    }

    pub fn handle_comments(&self) {
        for comment in self.comments.iter() {
            println!("{}", comment);
        }
    }

    pub fn handle_texts(&self) {
        for text in self.texts.iter() {
            println!("{}", text);
        }
    }

    pub fn handle_inputs(&self) {
        for input in self.inputs.iter() {
            println!("{}", input);
        }
    }
}
