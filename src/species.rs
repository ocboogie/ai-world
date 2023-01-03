use std::mem::swap;

use rand::prelude::SliceRandom;
use rand::Rng;

use crate::{
    environment::Environment, genome::Genome, innovation_record::InnovationRecord,
    organism::Organism,
};

const MUTATION_PROB: f64 = 0.2;
const COMPATIBILITY_THRESHOLD: f32 = 4.0;
const INTERSPECIE_MATE_PROB: f64 = 0.003;
const SURVIVAL_THRESHOLD: f32 = 0.2;

#[derive(Default, Debug)]
pub struct Species<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub representative: Genome<INPUT_SZ, OUTPUT_SZ>,
    pub members: Vec<Organism<INPUT_SZ, OUTPUT_SZ>>,
    pub average_fitness: f32,
    pub champion: Organism<INPUT_SZ, OUTPUT_SZ>,
    pub age: usize,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> Species<INPUT_SZ, OUTPUT_SZ> {
    pub fn evaluate(&mut self, env: &mut impl Environment<INPUT_SZ, OUTPUT_SZ>) {
        if self.members.is_empty() {
            return;
        }

        let mut fitness_sum: f32 = 0.0;
        let mut champion_fitness: f32 = f32::NEG_INFINITY;
        let mut champion: usize = 0;

        for (i, organism) in self.members.iter_mut().enumerate() {
            let fitness = env.evaluate(&organism.genome);
            organism.fitness = fitness;
            fitness_sum += fitness;
            if fitness > champion_fitness {
                champion_fitness = fitness;
                champion = i;
            }
        }

        self.champion = self.members[champion].clone();
        self.average_fitness = fitness_sum / self.members.len() as f32;
    }

    pub fn cull(&mut self) {
        self.members.sort_by(|a, b| b.fitness.total_cmp(&a.fitness));

        // dbg!(self.members.iter().map(|f| f.fitness).collect::<Vec<_>>());

        self.members
            .truncate((self.members.len() as f32 * SURVIVAL_THRESHOLD).ceil() as usize);
    }

    pub fn reproduce(
        &mut self,
        rng: &mut impl Rng,
        offspring: usize,
        organisms: &[Organism<INPUT_SZ, OUTPUT_SZ>],
    ) {
        self.age += 1;

        let old_members = self.members.clone();

        self.members.resize_with(offspring, || {
            let organism = old_members.choose(rng).unwrap();
            let other_organism = if rng.gen_bool(INTERSPECIE_MATE_PROB) {
                organisms.choose(rng).unwrap()
            } else {
                old_members.choose(rng).unwrap()
            };

            Organism::crossover(organism, other_organism, rng)
        });
        println!(
            "{}, {}, {}",
            offspring,
            self.members.len(),
            self.average_fitness
        );
    }

    pub fn mutate(
        &mut self,
        rng: &mut impl Rng,
        innovation_record: &mut InnovationRecord<INPUT_SZ, OUTPUT_SZ>,
    ) {
        for organism in &mut self.members {
            if rng.gen_bool(MUTATION_PROB) {
                organism.genome.mutate(rng, innovation_record)
            }
        }
    }

    pub fn from_representative(organism: Organism<INPUT_SZ, OUTPUT_SZ>) -> Self {
        Self {
            representative: organism.genome.clone(),
            champion: organism.clone(),
            average_fitness: organism.fitness,
            members: vec![organism],
            age: 0,
        }
    }

    pub fn is_compatible(&self, genome: &Genome<INPUT_SZ, OUTPUT_SZ>) -> bool {
        self.representative.distance(genome) < COMPATIBILITY_THRESHOLD
    }
}
