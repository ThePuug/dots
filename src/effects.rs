use crate::common::coord::Coord;

#[derive(Debug)]
pub struct Effect {
    pub pos: Option<Coord>,
    pub typ: Option<EffectType>,
    pub val: Option<f32>
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub enum EffectType {
    OPACITY
    // X,
    // Y,
}
