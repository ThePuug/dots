use crate::common::dna::Dna;

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub enum Effect {
    ENERGY(u16),
    SEED(Dna),
}
