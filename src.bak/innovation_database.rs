use std::collections::HashMap;

#[derive(Default)]
pub struct InnovationDatabase {
    counter: usize,
    db: HashMap<(usize, usize), usize>,
}

impl InnovationDatabase {
    pub fn get(&mut self, in_neuron: usize, out_neuron: usize) -> usize {
        *self.db.entry((in_neuron, out_neuron)).or_insert_with(|| {
            let tmp = self.counter;
            self.counter += 1;
            tmp
        })
    }
}
