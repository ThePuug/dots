use rand::Rng;
use std::sync::mpsc::Sender;

use common::coord::Coord;
use action::Action;
use effects::{Effect,EffectType};

pub struct Dot {
    pub pos: Coord,
    pub color: [f32; 4]
}

impl Dot {
    pub fn act(&self) -> Action {
        let act: Action = rand::random();
        return act;
    }

    pub fn tick(&self, tx: Sender<Effect>) {
//        println!("doing TICK");
        if self.is_alive() {
            match self.act() {
                Action::DARKEN => {
                    match self.reach(1) {
                        Some(pos) => {
//                            println!("DARKEN {},{}",pos.x,pos.y);
                            tx.send(Effect {
                                pos: Some(pos),
                                effect: EffectType::OPACITY,
                                intensity: 0.1
                            });
                        },
                        None => {}
                    };
                },
                Action::LIGHTEN => {
                    match self.reach(1) {
                        Some(pos) => {
//                            println!("LIGHTEN {},{}",pos.x,pos.y);
                            tx.send(Effect {
                                pos: Some(pos),
                                effect: EffectType::OPACITY,
                                intensity: -0.1
                            });
                        },
                        None => {}
                    };
                },
                _ => {}
            };
        }
    }

    pub fn update(&mut self, tx: Sender<Effect>, property: EffectType, intensity: f32) {
        match property {
            EffectType::TICK => self.tick(tx),
            EffectType::OPACITY => self.color[3] += intensity
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