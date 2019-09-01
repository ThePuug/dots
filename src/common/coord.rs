use std::hash::{Hash,Hasher};

#[derive(Copy, Clone)]
pub struct Coord {
    pub x: f64,
    pub y: f64
}

impl PartialEq for Coord {
    fn eq(&self, other: &Self) -> bool { self.x as u32 == other.x as u32 && self.y as u32 == other.y as u32 }
}

impl Eq for Coord {}

impl Hash for Coord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.x as u32).hash(state);
        (self.y as u32).hash(state);
    }
}