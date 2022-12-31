use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Node<const INPUT_SZ: usize, const OUTPUT_SZ: usize>(pub usize);

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> Deref for Node<INPUT_SZ, OUTPUT_SZ> {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> DerefMut for Node<INPUT_SZ, OUTPUT_SZ> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const INPUT_SZ: usize, const OUTPUT_SZ: usize> Node<INPUT_SZ, OUTPUT_SZ> {
    pub fn is_bias(&self) -> bool {
        self.0 == 0
    }

    pub fn is_input(&self) -> bool {
        self.0 >= 1 && self.0 < 1 + INPUT_SZ
    }

    pub fn is_output(&self) -> bool {
        self.0 >= 1 + INPUT_SZ && self.0 < 1 + INPUT_SZ + OUTPUT_SZ
    }

    pub fn is_hidden(&self) -> bool {
        self.0 >= 1 + INPUT_SZ + OUTPUT_SZ
    }
}
