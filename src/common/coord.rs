use std::hash::{Hash,Hasher};

#[derive(Copy, Clone)]
pub struct Coord {
    pub x: f64,
    pub y: f64
}

impl PartialEq for Coord {
    fn eq(&self, other: &Coord) -> bool {
        return self.x as i32 == other.x as i32 && self.y as i32 == other.y as i32
    }
}

impl Eq for Coord {}

impl Hash for Coord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.x as i32).hash(state);
        (self.y as i32).hash(state);
    }
}