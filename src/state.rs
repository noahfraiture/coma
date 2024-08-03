use crate::topology::Node;
use colored::Colorize;
use std::{
    collections::{HashSet, LinkedList},
    fmt,
    sync::{Arc, Mutex},
};

pub struct State {
    pub current_depth: i32,
    visited: HashSet<Arc<Node>>,
    layers: LinkedList<Mutex<Vec<Arc<Node>>>>,
    current_layer: Mutex<Vec<Arc<Node>>>,
}

impl State {
    pub fn new(root: Arc<Node>) -> Result<Self, Box<dyn std::error::Error>> {
        // Queue of vector of the discovered link at the current depth
        // Each node of the linkedlist is a depth
        let mut layers: LinkedList<Mutex<Vec<Arc<Node>>>> = LinkedList::new();
        layers.push_back(Mutex::new(vec![root]));

        // This will never be used and could be None
        let current_layer = Mutex::new(vec![]);

        Ok(State {
            current_depth: 0,
            visited: HashSet::new(),
            layers,
            current_layer,
        })
    }

    pub fn pop_layer(&mut self) -> Option<()> {
        self.current_layer = self.layers.pop_front()?;
        Some(())
    }

    pub fn next_url(&self) -> Result<Option<Arc<Node>>, StateError> {
        if let Ok(mut layer) = self.current_layer.lock() {
            Ok(layer.pop())
        } else {
            Err(StateError::Lock)
        }
    }

    pub fn add_to_next_layer(
        &mut self,
        links: Vec<Arc<Node>>,
    ) -> std::result::Result<(), StateError> {
        if self.layers.front().is_none() {
            self.layers.push_back(Mutex::new(vec![]));
        }
        if let Ok(mut layer) = self.layers.front().unwrap().lock() {
            layer.extend(links);
            Ok(())
        } else {
            Err(StateError::Message(String::from("Error to lock")))
        }
    }

    pub fn known(&mut self, node: &Arc<Node>) -> bool {
        !self.visited.insert(Arc::clone(node))
    }

    pub fn deeper(&mut self) {
        self.current_depth += 1;
    }
}

pub enum StateError {
    Lock,
    Message(String),
}

impl StateError {
    fn print(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::Lock => {
                write!(f, "{}: locking mutex problem", "State error".red())
            }
            StateError::Message(s) => {
                write!(f, "{}: {}", "State error".red(), s)
            }
        }
    }
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f)
    }
}

impl fmt::Debug for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f)
    }
}

impl std::error::Error for StateError {}
