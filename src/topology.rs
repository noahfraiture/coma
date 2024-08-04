use futures::future::join_all;
use std::sync::{Arc, Mutex, Weak};
use tokio::task;
use url::Url;

use crate::scrapy::crawl;

pub struct Node {
    pub id: String,
    pub url: Url,
    // Every node will own every images on the page
    // More logic that every node own a copy of the url to the image

    // Mutex is need to borrow mutability of Arc
    pub images: Mutex<Vec<Url>>,
    // Can't directly use scraper::node::{Comment, Text} since their aren't Send/Sync
    // Could try later to impl these trait
    pub comments: Mutex<Vec<String>>,
    pub texts: Mutex<Vec<String>>,
    pub children: Mutex<Vec<Arc<Node>>>,
    pub parents: Mutex<Vec<Weak<Node>>>,
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
            images: Mutex::new(Vec::new()),
            comments: Mutex::new(Vec::new()),
            texts: Mutex::new(Vec::new()),
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

    pub fn quantity_elements(&self) -> usize {
        self.images.lock().unwrap().len()
            + self.comments.lock().unwrap().len()
            + self.texts.lock().unwrap().len()
    }

    pub async fn handle_images(&self) {
        let mut tasks = Vec::new();
        for image in self.images.lock().unwrap().iter().map(|v| v.to_owned()) {
            let task = task::spawn(async move {
                // TODO return error
                crawl::download_img(&image).await.unwrap();
            });
            tasks.push(task);
        }
        join_all(tasks).await;
    }

    pub fn handle_comments(&self) {
        for comment in self.comments.lock().unwrap().iter() {
            println!("{}", comment);
        }
    }

    pub fn handle_texts(&self) {
        for text in self.texts.lock().unwrap().iter() {
            println!("{}", text);
        }
    }
}
