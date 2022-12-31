use crate::{gene::Gene, innovation_database::InnovationDatabase};
use rand::seq::SliceRandom;
use rand::Rng;
use std::ops::{Index, IndexMut};

const BIAS: f32 = 1.0;

const MUTATE_WEIGHTS_RATE: f32 = 0.80;
const PERTURB_WEIGHT_RATE: f32 = 0.90;
const NEW_NEURON_RATE: f32 = 0.03;
const NEW_GENE_RATE: f32 = 0.05;

#[derive(Debug)]
pub struct Genome<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub hidden_neurons: usize,
    pub genes: Vec<Gene>,
}

#[derive(Debug)]
struct GenomeRuntime<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub input: [f32; INPUT_SZ],
    pub output: [f32; OUTPUT_SZ],
    pub hidden: Vec<f32>,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> GenomeRuntime<INPUT_SZ, OUTPUT_SZ> {
    pub fn new(input: [f32; INPUT_SZ], hidden_neurons: usize) -> Self {
        Self {
            input,
            output: [f32::NAN; OUTPUT_SZ],
            hidden: vec![f32::NAN; hidden_neurons],
        }
    }
}

/// 0 = Bias
/// 1:INPUT_SZ + 1 = Input neurons
/// INPUT_SZ + 1:OUTPUT_SZ + INPUT_SZ + 1 = Output neurons
/// OUTPUT_SZ + INPUT_SZ + 1: = Hidden neuron
impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> Index<usize>
    for GenomeRuntime<INPUT_SZ, OUTPUT_SZ>
{
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        if index == 0 {
            return &BIAS;
        }

        if index - 1 < INPUT_SZ {
            return &self.input[index - 1];
        }

        if index - INPUT_SZ - 1 < OUTPUT_SZ {
            return &self.output[index - INPUT_SZ - 1];
        }

        &self.hidden[index - INPUT_SZ - OUTPUT_SZ - 1]
    }
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> IndexMut<usize>
    for GenomeRuntime<INPUT_SZ, OUTPUT_SZ>
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index == 0 {
            panic!("Can't mutate virtual bias neuron which is at index 1");
        }

        if index - 1 < INPUT_SZ {
            return &mut self.input[index - 1];
        }

        if index - INPUT_SZ - 1 < OUTPUT_SZ {
            return &mut self.output[index - INPUT_SZ - 1];
        }

        &mut self.hidden[index - INPUT_SZ - OUTPUT_SZ - 1]
    }
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> Genome<INPUT_SZ, OUTPUT_SZ> {
    pub fn new() -> Self {
        Self {
            hidden_neurons: 0,
            genes: Vec::new(),
        }
    }

    pub fn size(&self) -> usize {
        INPUT_SZ + OUTPUT_SZ + self.hidden_neurons + 1
    }

    fn mutate_weights(&mut self, rng: &mut impl Rng) {
        for mut gene in self.genes.iter_mut() {
            if rng.gen::<f32>() < PERTURB_WEIGHT_RATE {
                gene.weight += rng.gen::<f32>() - 0.5;
            } else {
                gene.weight = rng.gen();
            }
        }
    }

    fn mutate_new_gene(&mut self, rng: &mut impl Rng, innovation_db: &mut InnovationDatabase) {
        let in_neuron: usize = rng.gen_range(0..self.size());
        let out_neuron: usize = rng.gen_range((INPUT_SZ + 1)..self.size());

        // The input neuron of a gene can't be an output neuron
        // TODO: It would be better to not let this happen in the first place
        // rather than returning if it does happen
        if in_neuron > INPUT_SZ + 1 && in_neuron < INPUT_SZ + OUTPUT_SZ + 1 {
            return;
        }

        if in_neuron == out_neuron {
            return;
        }

        // Find gene with same connecting neurons and ensure it is enabled if
        // it exists
        // TODO: Use a hashmap to find genes by connections
        if let Some(mut existing_gene) = self
            .genes
            .iter_mut()
            .find(|gene| gene.in_neuron == in_neuron && gene.out_neuron == out_neuron)
        {
            existing_gene.enabled = true;
            return;
        }

        // TODO: Disallow cycles

        self.genes.push(Gene {
            in_neuron,
            out_neuron,
            weight: rng.gen(),
            enabled: true,
            innovation_number: innovation_db.get(in_neuron, out_neuron),
        })
    }

    fn mutate_new_neuron(&mut self, rng: &mut impl Rng, innovation_db: &mut InnovationDatabase) {
        let new_neuron = self.size();

        if let Some(mut old_gene) = self.genes.choose_mut(rng) {
            old_gene.enabled = false;
            let in_neuron = old_gene.in_neuron;
            let out_neuron = old_gene.out_neuron;
            let weight = old_gene.weight;

            self.genes.push(Gene {
                in_neuron,
                out_neuron: new_neuron,
                weight: 1.0,
                enabled: true,
                innovation_number: innovation_db.get(in_neuron, new_neuron),
            });
            self.genes.push(Gene {
                in_neuron: new_neuron,
                out_neuron,
                weight,
                enabled: true,
                innovation_number: innovation_db.get(new_neuron, out_neuron),
            });
            self.hidden_neurons += 1;
        }
    }

    pub fn mutate(&mut self, rng: &mut impl Rng, innovation_db: &mut InnovationDatabase) {
        if rng.gen::<f32>() < MUTATE_WEIGHTS_RATE {
            self.mutate_weights(rng);
        }
        if rng.gen::<f32>() < NEW_GENE_RATE {
            self.mutate_new_gene(rng, innovation_db);
        }
        if rng.gen::<f32>() < NEW_NEURON_RATE {
            self.mutate_new_neuron(rng, innovation_db);
        }
    }

    fn compute_neuron(
        &self,
        target: usize,
        runtime: &mut GenomeRuntime<INPUT_SZ, OUTPUT_SZ>,
    ) -> f32 {
        if !runtime[target].is_nan() {
            return runtime[target];
        }

        let val = self
            .genes
            .iter()
            .filter(|connection| connection.out_neuron == target && connection.enabled)
            .map(|connection| {
                self.compute_neuron(connection.in_neuron, runtime) * connection.weight
            })
            .sum();

        runtime[target] = val;

        val
    }

    pub fn calculate<I, O>(&self, input: I) -> O
    where
        I: Into<[f32; INPUT_SZ]>,
        O: From<[f32; OUTPUT_SZ]>,
    {
        let mut runtime = GenomeRuntime::new(input.into(), self.hidden_neurons);

        for i in 0..OUTPUT_SZ {
            self.compute_neuron(i + INPUT_SZ + 1, &mut runtime);
        }

        runtime.output.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            Genome::<1, 1> {
                hidden_neurons: 1,
                genes: vec![
                    Gene {
                        in_neuron: 1,
                        out_neuron: 3,
                        weight: 0.5,
                        enabled: true,
                        innovation_number: 0,
                    },
                    Gene {
                        in_neuron: 3,
                        out_neuron: 2,
                        weight: 0.5,
                        enabled: true,
                        innovation_number: 1,
                    },
                ],
            }
            .calculate::<[f32; 1], [f32; 1]>([12.0]),
            [3.0]
        );

        assert_eq!(
            Genome::<1, 1> {
                hidden_neurons: 0,
                genes: vec![Gene {
                    in_neuron: 1,
                    out_neuron: 2,
                    weight: 0.5,
                    enabled: false,
                    innovation_number: 0,
                }]
            }
            .calculate::<[f32; 1], [f32; 1]>([12.0]),
            [0.0]
        );
    }
}
