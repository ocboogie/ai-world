use crate::genome::Genome;

#[derive(Default)]
pub struct Organism<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub fitness: f32,
    pub genome: Genome<INPUT_SZ, OUTPUT_SZ>,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> Organism<INPUT_SZ, OUTPUT_SZ> {
    pub fn new(genome: Genome<INPUT_SZ, OUTPUT_SZ>) -> Self {
        Self {
            fitness: 0.0,
            genome,
        }
    }
}
