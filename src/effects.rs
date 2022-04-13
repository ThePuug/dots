use crate::common::coord::Coord;

#[derive(Debug)]
pub struct Effect {
    pub pos: Option<Coord>,
    pub typ: Option<EffectType>,
    pub val: Option<[u8;8]>
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub enum EffectType {
    OPACITY
}
