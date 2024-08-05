
## Import statement
### Topology
We import the Node struct from the topology module into the current module. In Rust, the use keyword is used to bring items (such as structs, functions, or enums) into scope, allowing them to be used without specifying their full path every time.

In this case, the Node struct is being imported from the topology module. This means that any code within the current module can now refer to Node directly, without needing to prefix it with topology::. This can make the code more concise and readable.

Here it comes from the file topology.rs in the src directory, more specifically the declaration

```rust
pub struct Node {
    // all the fields
}
```

### Linked List
We import the LinkedList struct from the standard library's collections module into the current module. The LinkedList struct represents a doubly linked list, a data structure where each element has a reference to the next and previous elements in the list.

In state.rs, we have the following 

```rust
pub struct State {
    pub current_depth: i32,
    pub current_external: i32,
    visited: Vec<Arc<Mutex<Node>>>,
    layers: LinkedList<Vec<Arc<Mutex<Node>>>>,
    pub current_layer: Vec<Arc<Mutex<Node>>>,
}
```

The LinkedList struct is used to store layers of nodes in the State struct. This allows us to efficiently manage and access the layers of nodes in the state of the system.