use common::coord::Coord;
use dots::Dot;

use std::collections::HashMap;
use std::sync::{Arc,Mutex};

pub struct Scene {
    pub size: Coord,
    pub scale: u8,
    pub dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<Dot>>>>>
}

impl Scene {
    pub fn new(size: Coord, scale: u8) -> Scene {
        return Scene {
            size,
            scale,
            dots: Arc::new(Mutex::new(HashMap::new()))
        };
    }
}