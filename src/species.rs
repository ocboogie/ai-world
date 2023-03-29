use crate::{client::ClientId, evaluation::Evaluation, genome::Genome};

const COMPATIBILITY_THRESHOLD: f32 = 6.0;

pub type SpeciesId = usize;

#[derive(Debug, Clone)]
pub struct Species<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub representative: Genome<INPUT_SZ, OUTPUT_SZ>,
    pub members: Vec<ClientId>,
    pub age: usize,
    pub id: SpeciesId,
    pub max_fitness: f32,
    pub since_last_improvement: usize,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> Species<INPUT_SZ, OUTPUT_SZ> {
    pub fn is_compatible(&self, genome: &Genome<INPUT_SZ, OUTPUT_SZ>) -> bool {
        self.representative.distance(genome) < COMPATIBILITY_THRESHOLD
    }

    pub fn sort_by_fitness(&mut self, evaluation: &Evaluation) {
        self.members.sort_by(|member_a, member_b| {
            evaluation.fitness[member_b].total_cmp(&evaluation.fitness[member_a])
        });
    }

    //
    // pub fn reproduce(
    //     &mut self,
    //     rng: &mut impl Rng,
    //     offspring: usize,
    //     organisms: &[Organism<INPUT_SZ, OUTPUT_SZ>],
    // ) {
    //     self.age += 1;
    //
    //     let old_members = self.members.clone();
    //
    //     self.members.resize_with(offspring, || {
    //         let organism = old_members.choose(rng).unwrap();
    //         let other_organism = if rng.gen_bool(INTERSPECIE_MATE_PROB) {
    //             organisms.choose(rng).unwrap()
    //         } else {
    //             old_members.choose(rng).unwrap()
    //         };
    //
    //         Organism::crossover(organism, other_organism, rng)
    //     });
    //     println!(
    //         "{}, {}, {}",
    //         offspring,
    //         self.members.len(),
    //         self.average_fitness
    //     );
    // }
    //
    // pub fn mutate(
    //     &mut self,
    //     rng: &mut impl Rng,
    //     innovation_record: &mut InnovationRecord<INPUT_SZ, OUTPUT_SZ>,
    // ) {
    //     for organism in &mut self.members {
    //         if rng.gen_bool(MUTATION_PROB) {
    //             organism.genome.mutate(rng, innovation_record)
    //         }
    //     }
    // }
}
