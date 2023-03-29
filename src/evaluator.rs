use std::mem::swap;

use rand::{rngs::ThreadRng, thread_rng};

use crate::{
    environment::Environment, evaluation::Evaluation, innovation_record::InnovationRecord,
    population::Population, speciation::Speciation,
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
    pub last_speciation: Option<Speciation<INPUT_SZ, OUTPUT_SZ>>,
    pub last_evaluation: Option<Evaluation>,
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
            last_speciation: None,
            last_evaluation: None,
        }
    }

    pub fn evaluate_and_evolve(&mut self) {
        if let (Some(speciation), Some(evaluation)) =
            (&mut self.last_speciation, &mut self.last_evaluation)
        {
            self.population.evolve(
                evaluation,
                speciation,
                &mut self.rng,
                &mut self.innovation_record,
            );
        }

        self.last_speciation = Some(self.population.speciate(
            &mut self.rng,
            self.last_speciation.as_ref(),
            self.last_evaluation.as_ref(),
        ));

        self.last_evaluation = Some(self.population.evaluate(&mut self.env));
    }
}
