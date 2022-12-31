use std::collections::HashMap;

use crate::node::Node;

#[derive(Default)]
pub struct InnovationRecord<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    counter: usize,
    db: HashMap<(Node<INPUT_SZ, OUTPUT_SZ>, Node<INPUT_SZ, OUTPUT_SZ>), usize>,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> InnovationRecord<INPUT_SZ, OUTPUT_SZ> {
    pub fn get(
        &mut self,
        in_node: Node<INPUT_SZ, OUTPUT_SZ>,
        out_node: Node<INPUT_SZ, OUTPUT_SZ>,
    ) -> usize {
        *self.db.entry((in_node, out_node)).or_insert_with(|| {
            let tmp = self.counter;
            self.counter += 1;
            tmp
        })
    }
}
