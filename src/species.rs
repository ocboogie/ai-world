use crate::genome::Genome;

const COMPATIBILITY_THRESHOLD: f32 = 3.0;

struct Species<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    members: Vec<Genome<INPUT_SZ, OUTPUT_SZ>>,
    representative: usize,
    age: usize,
}
