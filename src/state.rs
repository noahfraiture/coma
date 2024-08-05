use crate::topology::Node;
use std::{
    collections::LinkedList,
    sync::{Arc, Mutex},
};

pub struct State {
    pub current_depth: i32,
    pub current_external: i32,
    visited: Vec<Arc<Mutex<Node>>>,
    layers: LinkedList<Vec<Arc<Mutex<Node>>>>,
    pub current_layer: Vec<Arc<Mutex<Node>>>,
}

impl State {
    pub fn new(root: Arc<Mutex<Node>>) -> Result<Self, Box<dyn std::error::Error>> {
        // Queue of vector of the discovered link at the current depth
        // Each node of the linkedlist is a depth
        let mut layers: LinkedList<Vec<Arc<Mutex<Node>>>> = LinkedList::new();
        layers.push_back(vec![root]);

        // This will never be used and could be None
        let current_layer = Vec::new();

        Ok(State {
            current_depth: 0,
            current_external: 0,
            visited: Vec::new(),
            layers,
            current_layer,
        })
    }

    pub fn pop_layer(&mut self) -> Option<()> {
        self.current_layer = self.layers.pop_front()?;
        Some(())
    }

    pub fn add_to_next_layer(&mut self, links: Vec<Arc<Mutex<Node>>>) {
        if self.layers.front().is_none() {
            self.layers.push_back(Vec::new());
        }
        self.layers.front_mut().unwrap().extend(links);
    }

    pub fn known(&mut self, node: &Arc<Mutex<Node>>) -> bool {
        for visited_node in self.visited.iter() {
            if visited_node.lock().unwrap().eq(&node.lock().unwrap()) {
                return true;
            }
        }
        self.visited.push(Arc::clone(node));
        false
    }
}
