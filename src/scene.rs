use common::coord::Coord;
use dots::Dot;
use physics::Physics;

use std::collections::HashMap;
use std::sync::{Arc,Mutex};

pub struct Scene {
    pub size: Coord,
    pub scale: u8,
    pub physics: Physics
}

impl Scene {
    pub fn new(size: Coord, scale: u8, dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<Dot>>>>>) -> Scene {
        let physics;
        {
            let dots = dots.clone();
            physics = Physics::new(dots);
        };

        return Scene {
            size,
            scale,
            physics
        };
    }
}