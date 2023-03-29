use std::collections::HashMap;

use crate::{client::ClientId, species::Species};

#[derive(Clone)]
pub struct Evaluation {
    pub fitness: HashMap<ClientId, f32>,
    pub adjusted: bool,
}

impl Evaluation {
    pub fn species_average_fitness<const INPUT_SZ: usize, const OUTPUT_SZ: usize>(
        &self,
        species: &Species<INPUT_SZ, OUTPUT_SZ>,
    ) -> f32 {
        species
            .members
            .iter()
            .map(|id| {
                self.fitness
                    .get(id)
                    .expect("Could not find member in species")
            })
            .sum::<f32>()
            / species.members.len() as f32
    }

    pub fn species_max_fitness<const INPUT_SZ: usize, const OUTPUT_SZ: usize>(
        &self,
        species: &Species<INPUT_SZ, OUTPUT_SZ>,
    ) -> f32 {
        species
            .members
            .iter()
            .map(|id| {
                *self
                    .fitness
                    .get(id)
                    .expect("Could not find member in species")
            })
            .max_by(|a, b| a.total_cmp(b))
            .expect("Can't find max fitness")
    }

    pub fn species_average_adjusted_fitness<const INPUT_SZ: usize, const OUTPUT_SZ: usize>(
        &self,
        species: &Species<INPUT_SZ, OUTPUT_SZ>,
    ) -> f32 {
        // For more info, see https://neat-python.readthedocs.io/en/latest/_modules/reproduction.html
        let min_fitness = self
            .fitness
            .values()
            .min_by(|a, b| a.total_cmp(&b))
            .unwrap();
        let max_fitness = self
            .fitness
            .values()
            .max_by(|a, b| a.total_cmp(&b))
            .unwrap();

        let fitness_range = (max_fitness - min_fitness).max(1.0);

        let average_species_fitness = self.species_average_fitness(species);

        (average_species_fitness - min_fitness) / fitness_range
    }

    pub fn champion(&self) -> (ClientId, f32) {
        self.fitness
            .iter()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(id, fitness)| (*id, *fitness))
            .expect("Can't find champion of empty evaluation")
    }

    pub fn average_fitness(&self) -> f32 {
        self.fitness.values().sum::<f32>() / self.fitness.len() as f32
    }

    pub fn species_champion<const INPUT_SZ: usize, const OUTPUT_SZ: usize>(
        &self,
        species: &Species<INPUT_SZ, OUTPUT_SZ>,
    ) -> (ClientId, f32) {
        species
            .members
            .iter()
            .map(|id| {
                (
                    *id,
                    *self
                        .fitness
                        .get(id)
                        .expect("Could not find member in species"),
                )
            })
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .expect("Can't find champion of empty species")
    }
}
