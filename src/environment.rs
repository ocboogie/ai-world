use crate::genome::Genome;

pub trait Environment<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    fn evaluate(&mut self, genome: &Genome<INPUT_SZ, OUTPUT_SZ>) -> f32;
}
