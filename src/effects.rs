use common::coord::Coord;

pub struct Effect {
    pub pos: Option<Coord>,
    pub effect: EffectType,
    pub intensity: f32
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum EffectType {
    TICK,
    OPACITY
}