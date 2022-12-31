#[derive(Debug)]
pub struct Gene {
    pub in_neuron: usize,
    pub out_neuron: usize,
    pub weight: f32,
    pub enabled: bool,
    pub innovation_number: usize,
}
