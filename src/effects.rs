use common::coord::Coord;

pub struct Effect {
    pub pos: Option<Coord>,
    pub typ: Option<EffectType>,
    pub val: Option<f32>
}

#[derive(PartialEq, Eq, Copy, Clone, Hash)]
pub enum EffectType {
    TICK,
    OPACITY,
    X,
    Y,
}
