use common::coord::Coord;

pub struct Effect {
    pub pos: Coord,
    pub effect: EffectType,
    pub intensity: f32
}

#[derive(PartialEq, Eq)]
pub enum EffectType {
    OPACITY
}