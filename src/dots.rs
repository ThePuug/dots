use std::sync::Arc;
use std::sync::mpsc::Sender;

use common::coord::Coord;
use action::Action;
use effects::{Effect,EffectType};
use neuralnets::{INeuralNet};

pub struct Dot<TNeuralNet> 
    where TNeuralNet : INeuralNet {
    pub pos: Coord,
    pub color: [f32; 4],
    neuralnet: TNeuralNet,
}

impl<TNeuralNet> Dot<TNeuralNet>
    where TNeuralNet : INeuralNet {
    pub fn new(pos: Coord, color: [f32;4], neuralnet: TNeuralNet)
        -> Dot<TNeuralNet> {
        Dot {
            pos,
            color,
            neuralnet: neuralnet,
        }
    }

    pub fn sense(&self) -> Arc<Vec<Arc<Effect>>> {
        return Arc::new(vec![
            Arc::new(Effect { pos: Some(Coord { x: self.pos.x-1.0, y: self.pos.y-0.0 }), typ: Some(EffectType::OPACITY), val: None }),
            Arc::new(Effect { pos: Some(Coord { x: self.pos.x-0.0, y: self.pos.y-1.0 }), typ: Some(EffectType::OPACITY), val: None }),
            Arc::new(Effect { pos: Some(Coord { x: self.pos.x+1.0, y: self.pos.y+0.0 }), typ: Some(EffectType::OPACITY), val: None }),
            Arc::new(Effect { pos: Some(Coord { x: self.pos.x+0.0, y: self.pos.y+1.0 }), typ: Some(EffectType::OPACITY), val: None })
        ]);
    }

    pub fn act(&mut self, tx: Sender<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>, cause: Arc<Vec<Arc<Effect>>>) {
        let mut inputs: Vec<bool> = Vec::new();
        for it in cause.iter() {
            if let Some(val) = it.val {
                inputs.push(if val >= 0.5 { true } else { false});
            } else { inputs.push(false) }
        }
        let outputs = self.neuralnet.forward(inputs);
        let mut x = 0;
        let mut y = 0;
        if outputs[0] { y -= 1; }
        if outputs[1] { x += 1; }
        if outputs[2] { y += 1; }
        if outputs[3] { x -= 1; }
        let action = if outputs[4] { Action::DARKEN } else { Action::IDLE };
        if action != Action::IDLE {
            match self.reach(x,y) {
                Some(pos) => {
                    tx.send((cause,Arc::new(Effect {
                        pos: Some(pos),
                        typ: Some(EffectType::OPACITY),
                        val: Some(0.1)
                    }))).unwrap()
                },
                None => {}
            };
        };
    }

    pub fn tick(&self, tx: Sender<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>) {
        // age every tick
        tx.send((
                Arc::new(vec![Arc::new(Effect { pos: None, typ: Some(EffectType::TICK), val: Some(1.0)})]),
                Arc::new(Effect { pos: Some(self.pos), typ: Some(EffectType::OPACITY), val: Some(-0.01)}))
            ).unwrap();

        // alive dots sense every tick
        if self.is_alive() {
            tx.send((self.sense(),Arc::new(Effect { pos: Some(self.pos), typ: None, val: None }))).unwrap()
        }
    }

    pub fn update(&mut self, tx: Sender<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>, effect: Arc<Effect>) {
        match effect.typ {
            Some(typ) => {
                match typ {
                    EffectType::TICK => self.tick(tx),
                    EffectType::OPACITY => {
                        self.color[3] += effect.val.unwrap();
                        if self.color[3] < 0.0 { self.color[3] = 0.0 }
                        if self.color[3] > 1.0 { self.color[3] = 1.0 }
                    }
                }
            },
            None => {}
        }
    }


    fn is_alive(&self) -> bool {
        return self.color[3] >= 0.5;
    }

    fn reach(&self, offset_x: i8, offset_y: i8) -> Option<Coord> {
        let x = self.pos.x + offset_x as f64;
        let y = self.pos.y + offset_y as f64;
        if 0.0 > x || x >= 50.0 || 0.0 > y || y >= 50.0 { return None; }
        return Some(Coord { x: x, y: y });
    }
}