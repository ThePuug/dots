use std::sync::mpsc;

use common::coord::Coord;
use physics::Physics;

pub struct Scene {
    pub size: Coord,
    pub scale: u8,
    pub physics: Physics,    
}

impl Scene {
    pub fn new(size: Coord, scale: u8) -> Scene {
        let physics = Physics::new();
        return Scene {
            size,
            scale,
            physics
        };
    }
}