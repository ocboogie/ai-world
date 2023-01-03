use rand::{rngs::ThreadRng, thread_rng};

use crate::{
    environment::Environment, innovation_record::InnovationRecord, population::Population,
};

pub struct Evaluator<
    const INPUT_SZ: usize,
    const OUTPUT_SZ: usize,
    E: Environment<INPUT_SZ, OUTPUT_SZ>,
> {
    pub env: E,
    pub innovation_record: InnovationRecord<INPUT_SZ, OUTPUT_SZ>,
    pub population: Population<INPUT_SZ, OUTPUT_SZ>,
    pub rng: ThreadRng,
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize, E: Environment<INPUT_SZ, OUTPUT_SZ>>
    Evaluator<INPUT_SZ, OUTPUT_SZ, E>
{
    pub fn new(env: E, target_size: usize) -> Self {
        let mut rng = thread_rng();
        let mut innovation_record = InnovationRecord::default();

        Self {
            population: Population::new(&mut rng, &mut innovation_record, target_size),
            env,
            innovation_record,
            rng,
        }
    }

    pub fn evaluate_and_evolve(&mut self) {
        if self.population.generation != 0 {
            self.population
                .evolve(&mut self.rng, &mut self.innovation_record);
        }

        self.population.speciate(&mut self.rng);

        self.population.evaluate(&mut self.env);
    }
}
