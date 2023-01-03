use rand::Rng;

use crate::{environment::Environment, genome::Genome};

#[derive(Default, Clone, Debug)]
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

    pub fn crossover(
        organism_1: &Organism<INPUT_SZ, OUTPUT_SZ>,
        organism_2: &Organism<INPUT_SZ, OUTPUT_SZ>,
        rng: &mut impl Rng,
    ) -> Organism<INPUT_SZ, OUTPUT_SZ> {
        let genome = if organism_1.fitness > organism_2.fitness {
            Genome::crossover(&organism_1.genome, &organism_2.genome, rng)
        } else {
            Genome::crossover(&organism_2.genome, &organism_1.genome, rng)
        };

        Organism::new(genome)
    }
}
