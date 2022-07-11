use crate::common::coord::Coord;
use crate::common::dna::Dna;

#[derive(Copy, Clone, Debug)]
pub enum Effect {
    ENERGY(f32, Option<[f32; 3]>, Option<Coord>),
    SEED(Dna),
}
