use std::sync::Arc;

use dots::Dot;
use common::coord::Coord;

pub struct Scene {
    pub dots: Vec<Arc<Dot>>,
    pub size: Coord,
    pub scale: u8
}

impl Scene {
    pub fn new(size: Coord, scale: u8) -> Scene {
        let vec = Vec::with_capacity((size.x * size.y) as usize);
        Scene {
            size,
            scale,
            dots: vec
        }
    }
}