use rand::Rng;

use crate::{
    environment::Environment, genome::Genome, innovation_record::InnovationRecord,
    organism::Organism,
};

const MUTATION_PROB: f64 = 0.25;
const COMPATIBILITY_THRESHOLD: f32 = 3.0;

pub struct Species<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub representative: Genome<INPUT_SZ, OUTPUT_SZ>,
    pub members: Vec<Organism<INPUT_SZ, OUTPUT_SZ>>,
    pub average_fitness: f32,
    pub champion: Genome<INPUT_SZ, OUTPUT_SZ>,
    pub age: usize,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> Species<INPUT_SZ, OUTPUT_SZ> {
    pub fn evaluate(&mut self, env: &mut impl Environment<INPUT_SZ, OUTPUT_SZ>) {
        if self.members.is_empty() {
            panic!("Tried to evaluate empty species!");
        }

        let mut fitness_sum: f32 = 0.0;
        let mut champion_fitness: f32 = f32::NEG_INFINITY;
        let mut champion: usize = 0;
        let members = self.members.len() as f32;

        for (i, organism) in self.members.iter_mut().enumerate() {
            let fitness = env.evaluate(&organism.genome) / members;
            organism.fitness = fitness;
            fitness_sum += fitness;
            if fitness > champion_fitness {
                champion_fitness = fitness;
                champion = i;
            }
        }

        self.champion = self.members[champion].genome.clone();
        self.average_fitness = fitness_sum / self.members.len() as f32;
    }

    pub fn evolve(
        &mut self,
        rng: &mut impl Rng,
        innovation_record: &mut InnovationRecord<INPUT_SZ, OUTPUT_SZ>,
        offspring: usize,
    ) {
        self.age += 1;
        self.members.sort_by(|a, b| a.fitness.total_cmp(&b.fitness));
        self.members.truncate(offspring);

        for organism in self.members.iter_mut() {
            if rng.gen_bool(MUTATION_PROB) {
                organism.genome.mutate(rng, innovation_record)
            } else {
                organism.genome = Genome::crossover(&self.champion, &organism.genome, rng);
            }
        }
    }

    pub fn from_representative(organism: Organism<INPUT_SZ, OUTPUT_SZ>) -> Self {
        Self {
            // TODO: Ew, clone
            // Not a problem of performance but design.
            // We shouldn't need a clone here.
            representative: organism.genome.clone(),
            champion: organism.genome.clone(),
            average_fitness: organism.fitness,
            members: vec![organism],
            age: 0,
        }
    }

    pub fn is_compatible(&self, genome: &Genome<INPUT_SZ, OUTPUT_SZ>) -> bool {
        self.representative.distance(genome) < COMPATIBILITY_THRESHOLD
    }
}
