use rand::Rng;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use common::coord::Coord;
use action::Action;
use effects::{Effect,EffectType};

pub struct Dot {
    pub pos: Coord,
    pub color: [f32; 4]
}

impl Dot {
    pub fn new(pos: Coord, color: [f32;4]) -> Dot {
        return Dot {
            pos,
            color
        };
    }

    pub fn sense(&self) -> Arc<Vec<Arc<Effect>>> {
        return Arc::new(vec![
            Arc::new(Effect { pos: Some(Coord { x: self.pos.x-1.0, y: self.pos.y-0.0 }), typ: Some(EffectType::OPACITY), val: None }),
            Arc::new(Effect { pos: Some(Coord { x: self.pos.x-0.0, y: self.pos.y-1.0 }), typ: Some(EffectType::OPACITY), val: None }),
            Arc::new(Effect { pos: Some(Coord { x: self.pos.x+1.0, y: self.pos.y+0.0 }), typ: Some(EffectType::OPACITY), val: None }),
            Arc::new(Effect { pos: Some(Coord { x: self.pos.x+0.0, y: self.pos.y+1.0 }), typ: Some(EffectType::OPACITY), val: None })
        ]);
    }

    pub fn act(&self, cause: Arc<Vec<Arc<Effect>>>, tx: Sender<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>) {
        let act: Action = rand::random();
        match act {
            Action::DARKEN => {
                match self.reach(1) {
                    Some(pos) => {
                        match tx.send((cause,Arc::new(Effect {
                            pos: Some(pos),
                            typ: Some(EffectType::OPACITY),
                            val: Some(0.1)
                        }))) {
                            Ok(_) => {},
                            Err(msg) => println!("{}",msg)
                        }
                    },
                    None => {}
                };
            },
            Action::LIGHTEN => {
                match self.reach(1) {
                    Some(pos) => {
                        match tx.send((cause,Arc::new(Effect {
                            pos: Some(pos),
                            typ: Some(EffectType::OPACITY),
                            val: Some(-0.1)
                        }))) {
                            Ok(_) => {},
                            Err(msg) => println!("{}",msg)
                        }
                    },
                    None => {}
                };
            },
            _ => {}
        };
    }

    pub fn tick(&self, tx: Sender<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>) {
        if self.is_alive() {
            let cause = self.sense();
            match tx.send((cause,Arc::new(Effect { pos: Some(self.pos), typ: None, val: None }))) {
                Ok(_) => {},
                Err(msg) => println!("{}",msg)
            }
        }
    }

    pub fn update(&mut self, tx: Sender<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>, effect: Arc<Effect>) {
        match effect.typ {
            Some(typ) => {
                match typ {
                    EffectType::TICK => self.tick(tx),
                    EffectType::OPACITY => self.color[3] += effect.val.unwrap()
                }
            },
            None => {}
        }
    }


    fn is_alive(&self) -> bool {
        return self.color[3] >= 0.5;
    }

    fn reach(&self, dist: u8) -> Option<Coord> {
        let mut rng = rand::thread_rng();
        let x = self.pos.x + rng.gen_range(dist as i8 * -1, (dist+1) as i8) as f64;
        let y = self.pos.y + rng.gen_range(dist as i8 * -1, (dist+1) as i8) as f64;
        if 0.0 > x || x >= 50.0 || 0.0 > y || y >= 50.0 || (x == self.pos.x && y == self.pos.y) { return None; }
        return Some(Coord { x: x, y: y });
    }
}