use crate::{
    environment::Environment,
    evaluation::Evaluation,
    genome::Genome,
    innovation_record::InnovationRecord,
    speciation::Speciation,
    species::{Species, SpeciesId},
};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{collections::HashMap, iter};

const INTERSPECIE_MATE_PROB: f64 = 0.003;
const SURVIVAL_THRESHOLD: f32 = 0.2;
const MUTATION_PROB: f64 = 0.2;
const STAGNANT_THRESHOLD: usize = 15;

#[derive(Clone)]
pub struct Population<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub members: Vec<Genome<INPUT_SZ, OUTPUT_SZ>>,
    pub target_size: usize,
    pub generation: usize,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> Population<INPUT_SZ, OUTPUT_SZ> {
    pub fn new(
        rng: &mut impl Rng,
        innovation_record: &mut InnovationRecord<INPUT_SZ, OUTPUT_SZ>,
        target_size: usize,
    ) -> Self {
        Self {
            generation: 0,
            target_size,
            members: (0..target_size)
                .map(|_| Genome::new_random_initial(rng, innovation_record))
                .collect(),
        }
    }

    pub fn evaluate(&mut self, env: &mut impl Environment<INPUT_SZ, OUTPUT_SZ>) -> Evaluation {
        Evaluation {
            fitness: self
                .members
                .iter_mut()
                .map(|member| env.evaluate(member))
                .enumerate()
                .collect(),
            adjusted: false,
        }
    }

    // See https://neat-python.readthedocs.io/en/latest/_modules/reproduction.html
    // Method compute_spawn
    fn compute_offspring(
        &self,
        evaluation: &Evaluation,
        speciation: &Speciation<INPUT_SZ, OUTPUT_SZ>,
    ) -> HashMap<SpeciesId, usize> {
        let total_average_species_adjusted_fitnesss = speciation
            .species
            .values()
            .map(|species| evaluation.species_average_adjusted_fitness(species))
            .sum::<f32>();

        let mut offspring: HashMap<SpeciesId, usize> = speciation
            .species
            .values()
            .map(|species| {
                let new_size = if total_average_species_adjusted_fitnesss > 0.0 {
                    evaluation.species_average_adjusted_fitness(species)
                        / total_average_species_adjusted_fitnesss
                        * self.target_size as f32
                } else {
                    1.0
                };
                let prev_size = species.members.len();

                let d = (new_size - prev_size as f32) * 0.5;
                let c = d.round() as isize;
                let final_size = if c.abs() > 0 {
                    prev_size.saturating_add_signed(c)
                } else if d > 0.0 {
                    prev_size + 1
                } else if d < 0.0 {
                    prev_size - 1
                } else {
                    prev_size
                };

                (species.id, final_size)
            })
            .collect();

        let total_offspring = offspring.values().sum::<usize>();

        let norm = self.target_size as f32 / total_offspring as f32;

        for offspring in offspring.values_mut() {
            *offspring = (((*offspring as f32) * norm).round() as usize).max(1);
        }

        offspring
    }

    fn breed(&self, evaluation: &Evaluation, selection: &[usize]) -> Genome<INPUT_SZ, OUTPUT_SZ> {
        // TODO: I usually like to pass down rngs, but there were some type
        // errors
        let mut rng = thread_rng();

        let parent_1 = *selection.choose(&mut rng).unwrap();
        let parent_2 = if rng.gen_bool(INTERSPECIE_MATE_PROB) {
            // FIXME: This should only choose from the survivors
            rng.gen_range(0..self.members.len())
        } else {
            *selection.choose(&mut rng).unwrap()
        };

        if evaluation.fitness[&parent_1] > evaluation.fitness[&parent_2] {
            Genome::crossover(&self.members[parent_1], &self.members[parent_2], &mut rng)
        } else {
            Genome::crossover(&self.members[parent_2], &self.members[parent_1], &mut rng)
        }
    }

    pub fn kill_stagnant_species(&mut self, speciation: &mut Speciation<INPUT_SZ, OUTPUT_SZ>) {
        speciation
            .species
            .retain(|_, species| species.since_last_improvement < STAGNANT_THRESHOLD)
        // speciation
        //     .species
        //     .retain(|_, species| species.age == 0 || species.members.len() > 1);
    }

    pub fn reproduce(
        &mut self,
        evaluation: &Evaluation,
        speciation: &mut Speciation<INPUT_SZ, OUTPUT_SZ>,
    ) {
        let offspring = self.compute_offspring(evaluation, speciation);
        // let total_average_species_adjusted_fitnesss = speciation
        //     .species
        //     .values()
        //     .map(|species| evaluation.species_average_adjusted_fitness(species))
        //     .sum::<f32>();

        let new_members = speciation
            .species
            .values_mut()
            .flat_map(|species| {
                species.sort_by_fitness(evaluation);
                let survivor_count =
                    (species.members.len() as f32 * SURVIVAL_THRESHOLD).ceil() as usize;

                // There should be at least two parents
                let survivor_count = survivor_count.max(2).min(species.members.len());

                let survivors = &species.members[..survivor_count];

                // let average_adjusted_fitness = evaluation.species_average_adjusted_fitness(species);
                let offspring = offspring[&species.id];
                // let offspring = ((average_adjusted_fitness
                //     / total_average_species_adjusted_fitnesss)
                //     * self.target_size as f32)
                //     .round() as usize;

                let champion_id = evaluation.species_champion(species).0;
                let champion_genome = self.members[champion_id].clone();

                iter::repeat_with(|| self.breed(evaluation, survivors))
                    .take(offspring - 1)
                    .chain(iter::once(champion_genome))
            })
            .collect::<Vec<_>>();

        self.members = new_members;
    }

    fn mutate(
        &mut self,
        rng: &mut impl Rng,
        innovation_record: &mut InnovationRecord<INPUT_SZ, OUTPUT_SZ>,
    ) {
        for member in &mut self.members {
            if rng.gen_bool(MUTATION_PROB) {
                member.mutate(rng, innovation_record);
            }
        }
    }
    pub fn evolve(
        &mut self,
        evaluation: &Evaluation,
        speciation: &mut Speciation<INPUT_SZ, OUTPUT_SZ>,
        rng: &mut impl Rng,
        innovation_record: &mut InnovationRecord<INPUT_SZ, OUTPUT_SZ>,
    ) {
        self.kill_stagnant_species(speciation);
        self.reproduce(evaluation, speciation);
        self.mutate(rng, innovation_record);
    }

    pub fn speciate(
        &mut self,
        rng: &mut impl Rng,
        last_speciation: Option<&Speciation<INPUT_SZ, OUTPUT_SZ>>,
        last_evaluation: Option<&Evaluation>,
    ) -> Speciation<INPUT_SZ, OUTPUT_SZ> {
        let mut member_map = HashMap::new();
        let mut species: HashMap<_, Species<INPUT_SZ, OUTPUT_SZ>> = HashMap::new();
        self.generation += 1;

        for (member_id, member) in self.members.iter().enumerate() {
            if let Some(last_species) =
                last_speciation.and_then(|Speciation { species, .. }| {
                    species
                        .values()
                        .find(|species| species.is_compatible(&member))
                })
            {
                if let Some(compatible_species) = species.get_mut(&last_species.id) {
                    compatible_species.members.push(member_id);
                } else {
                    let max_fitness = last_evaluation
                        .map(|eval| eval.species_max_fitness(last_species))
                        .unwrap_or(last_species.max_fitness);

                    species.insert(
                        last_species.id,
                        Species {
                            representative: member.clone(),
                            members: vec![member_id],
                            id: last_species.id,
                            age: last_species.age + 1,
                            max_fitness: last_species.max_fitness.max(max_fitness),
                            since_last_improvement: if max_fitness > last_species.max_fitness {
                                0
                            } else {
                                last_species.since_last_improvement + 1
                            },
                        },
                    );
                }

                member_map.insert(member_id, last_species.id);
            } else {
                let species_id = rng.gen();

                species.insert(
                    species_id,
                    Species {
                        representative: member.clone(),
                        members: vec![member_id],
                        id: species_id,
                        age: 0,
                        max_fitness: 0.0,
                        since_last_improvement: 0,
                    },
                );

                member_map.insert(member_id, species_id);
            }
        }

        Speciation {
            species,
            member_map,
        }
    }
}
