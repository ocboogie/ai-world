use std::hash::{Hash, Hasher};

use crate::node::Node;

#[derive(Debug, Clone)]
pub struct Connection<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub in_node: Node<INPUT_SZ, OUTPUT_SZ>,
    pub out_node: Node<INPUT_SZ, OUTPUT_SZ>,
    pub weight: f32,
    pub enabled: bool,
    pub innovation_number: usize,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> Hash for Connection<INPUT_SZ, OUTPUT_SZ> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.in_node.hash(state);
        self.out_node.hash(state);
        self.weight.to_bits().hash(state);
        self.enabled.hash(state);
        self.innovation_number.hash(state);
    }
}
