use common::coord::Coord;
use dots::Dot;
use neuralnets::INeuralNet;

use std::collections::HashMap;
use std::sync::{Arc,Mutex};

pub struct Scene<TNeuralNet>
    where TNeuralNet : INeuralNet {
    pub size: Coord,
    pub scale: u8,
    pub dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<Dot<TNeuralNet>>>>>>
}

impl<TNeuralNet> Scene<TNeuralNet>
    where TNeuralNet : INeuralNet {
    pub fn new(size: Coord, scale: u8, dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<Dot<TNeuralNet>>>>>>) -> Scene<TNeuralNet> {
        return Scene {
            size,
            scale,
            dots
        };
    }
}