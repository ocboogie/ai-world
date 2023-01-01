use std::mem::swap;

use rand::Rng;

use crate::{environment::Environment, species::Species};

pub struct Population<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    // TODO: Could have species be a many-to-one relations, then the Species struct might not even
    // be needed
    pub species: Vec<Species<INPUT_SZ, OUTPUT_SZ>>,
    pub generation: usize,
    pub size: usize,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> Population<INPUT_SZ, OUTPUT_SZ> {
    pub fn evaluate(&mut self, env: &mut impl Environment<INPUT_SZ, OUTPUT_SZ>) {
        for species in &mut self.species {
            species.evaluate(env);
        }
    }

    pub fn evolve(&mut self, rng: &mut impl Rng) {
        self.speciate();
        self.generation += 1;

        let average_species_fitnesss = self
            .species
            .iter()
            .map(|species| species.average_fitness)
            .sum()
            / self.species.len() as f32;

        for species in self.species.iter_mut() {
            // See "Is there another way to do fitness sharing?"
            // http://www.cs.ucf.edu/~kstanley/neat.html#intro

            let offspring =
                ((species.average_fitness / average_species_fitnesss) * self.size as f32) as usize;

            species.evolve(rng, average_species_fitnesss);
        }
    }

    pub fn speciate(&mut self) {
        let organisms = self
            .species
            .iter_mut()
            .flat_map(|species| {
                let mut tmp = Vec::new();
                swap(&mut tmp, &mut species.members);
                tmp.into_iter()
            })
            .collect::<Vec<_>>();

        for organism in organisms.into_iter() {
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
    }
}
