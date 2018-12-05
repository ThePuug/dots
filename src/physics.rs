use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Sender,Receiver};
use std::thread;

use common::coord::Coord;
use dots::Dot;
use effects::Effect;

pub struct Engine {
    pub rx: Receiver<Effect>,
    pub tx: Sender<Effect>,
    pub dots: HashMap<Coord,Arc<Dot>>
}

impl Engine {
    pub fn new(tx: Sender<Effect>, rx: Receiver<Effect>) -> Engine {
        Engine {
            rx,
            tx,
            dots: HashMap::new()
        }
    }

    pub fn start(&self) {
        for msg in &self.rx {
            let dot = self.dots.entry(msg.pos).or_insert(Arc::new(Dot {
                pos: msg.pos,
                color:[0.0,0.0,0.0,0.0],
                tx: self.tx.clone()}));
            let property = msg.effect;
            let intensity = msg.intensity;
            thread::spawn(move || {
                dot.update(property, intensity);
            });
        }
    }
}