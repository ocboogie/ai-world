use crate::node::Node;

#[derive(Debug, Clone)]
pub struct Connection<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub in_node: Node<INPUT_SZ, OUTPUT_SZ>,
    pub out_node: Node<INPUT_SZ, OUTPUT_SZ>,
    pub weight: f32,
    pub enabled: bool,
    pub innovation_number: usize,
}
