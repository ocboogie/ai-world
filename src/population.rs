use std::{collections::HashMap, iter::once, mem::swap};

use rand::seq::SliceRandom;
use rand::Rng;

use crate::{
    environment::Environment, genome::Genome, innovation_record::InnovationRecord,
    organism::Organism, species::Species,
};

pub struct Population<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub species: Vec<Species<INPUT_SZ, OUTPUT_SZ>>,
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
            species: vec![{
                let rep = Genome::new_random_initial(rng, innovation_record);
                Species {
                    representative: rep.clone(),
                    members: (0..target_size - 1)
                        .map(|_| Organism::new(Genome::new_random_initial(rng, innovation_record)))
                        .chain(once(Organism::new(rep)))
                        .collect(),
                    ..Default::default()
                }
            }],
        }
    }

    pub fn size(&self) -> usize {
        self.species
            .iter()
            .map(|species| species.members.len())
            .sum()
    }

    fn get_organisms(&self) -> Vec<Organism<INPUT_SZ, OUTPUT_SZ>> {
        self.species
            .iter()
            .flat_map(|species| species.members.iter())
            .cloned()
            .collect()
    }

    pub fn evaluate(&mut self, env: &mut impl Environment<INPUT_SZ, OUTPUT_SZ>) {
        for species in &mut self.species {
            species.evaluate(env);
        }
    }

    pub fn evolve(
        &mut self,
        rng: &mut impl Rng,
        innovation_record: &mut InnovationRecord<INPUT_SZ, OUTPUT_SZ>,
    ) {
        for species in &mut self.species {
            species.cull();
        }
        // self.species.retain_mut(|species| {
        //     species.cull();
        //     species.members.len() > 1
        // });

        let organisms = self.get_organisms();

        let total_average_species_fitnesss = self
            .species
            .iter()
            .map(|species| species.average_fitness)
            .sum::<f32>();

        for species in self.species.iter_mut() {
            // See "Is there another way to do fitness sharing?"
            // http://www.cs.ucf.edu/~kstanley/neat.html#intro
            let offspring = ((species.average_fitness / total_average_species_fitnesss)
                * self.target_size as f32)
                .round() as usize;

            species.reproduce(rng, offspring, &organisms);
            species.mutate(rng, innovation_record);
        }
    }

    pub fn speciate(&mut self, rng: &mut impl Rng) {
        self.generation += 1;

        let organisms = self
            .species
            .iter_mut()
            .flat_map(|species| {
                let mut tmp = Vec::new();
                swap(&mut tmp, &mut species.members);
                tmp.into_iter()
            })
            .collect::<Vec<_>>();

        for organism in organisms {
            if let Some(species) = self
                .species
                .iter_mut()
                .find(|species| species.is_compatible(&organism.genome))
            {
                species.members.push(organism);
            } else {
                self.species.push(Species::from_representative(organism));
            }
        }

        self.species.retain(|species| !species.members.is_empty());

        for species in &mut self.species {
            species.representative = species.members.choose(rng).unwrap().genome.clone();
        }
    }
}
