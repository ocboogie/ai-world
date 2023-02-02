use std::collections::HashMap;

use crate::{
    client::ClientId,
    species::{Species, SpeciesId},
};

#[derive(Clone)]
pub struct Speciation<const INPUT_SZ: usize, const OUTPUT_SZ: usize> {
    pub species: HashMap<SpeciesId, Species<INPUT_SZ, OUTPUT_SZ>>,
    pub member_map: HashMap<ClientId, SpeciesId>,
}
