use std::convert::{From};
use std::marker::PhantomData;
use std::sync::{Arc};
use std::sync::mpsc::{Sender};

use common::coord::{Coord};
use action::{Action};
use effects::{Effect,EffectType};
use intelligence::{IIntelligence,IIntelligenceFactory};

pub trait IDot : Sized + Send + 'static {
    fn sense(&self) -> Arc<Vec<Arc<Effect>>>;
    fn act(&mut self, cause: Arc<Vec<Arc<Effect>>>);
    fn tick(&self);
    fn describe(&self, typ: EffectType) -> Option<f32>;
    fn apply_effect(&mut self, effect: (Arc<Vec<Arc<Effect>>>,Arc<Effect>));
    fn collides_with(&self, coord: Coord) -> bool;
}

pub trait IDotFactory<TDot,TIntelligence> : Sized
    where TDot : IDot {
    fn create(&self, pos: Coord, color: [f32;4]) -> TDot;
}

pub struct DotFactory<TIntelligenceFactory,TIntelligence> {
    intelligence_factory: TIntelligenceFactory,
    tx: Sender<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>,
    _t_intelligence: PhantomData<TIntelligence>
}

impl<TIntelligence,TIntelligenceFactory> DotFactory<TIntelligenceFactory,TIntelligence> 
    where TIntelligence : IIntelligence,
          TIntelligenceFactory : IIntelligenceFactory<TIntelligence> {
    pub fn new(intelligence_factory: TIntelligenceFactory, tx: Sender<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>) -> DotFactory<TIntelligenceFactory,TIntelligence> {
        DotFactory {
            intelligence_factory,
            tx,
            _t_intelligence: PhantomData,
        }
    }
}

impl<TIntelligence,TIntelligenceFactory,TDot> IDotFactory<TDot,TIntelligence> for DotFactory<TIntelligenceFactory,TIntelligence>
    where TIntelligence : IIntelligence,
          TIntelligenceFactory : IIntelligenceFactory<TIntelligence>,
          TDot : IDot + From<Dot<TIntelligence>> {
    fn create(&self, pos: Coord, color: [f32;4]) -> TDot {
        Dot::new(pos,color,self.intelligence_factory.create(),self.tx.clone()).into()
    }
}

pub struct Dot<TIntelligence> {
    pub pos: Coord,
    color: [f32; 4],
    intelligence: TIntelligence,
    tx: Sender<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>,
}

impl<TIntelligence> Dot<TIntelligence> 
    where TIntelligence : IIntelligence {
    fn new(pos: Coord, color: [f32;4], intelligence: TIntelligence, tx: Sender<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>)
        -> Dot<TIntelligence> {
        Dot {
            pos,
            color,
            intelligence,
            tx,
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

impl<TIntelligence> IDot for Dot<TIntelligence>
    where TIntelligence : IIntelligence {
    fn sense(&self) -> Arc<Vec<Arc<Effect>>> {
        return Arc::new(vec![
            Arc::new(Effect { pos: Some(Coord { x: self.pos.x-1.0, y: self.pos.y-0.0 }), typ: Some(EffectType::OPACITY), val: None }),
            Arc::new(Effect { pos: Some(Coord { x: self.pos.x-0.0, y: self.pos.y-1.0 }), typ: Some(EffectType::OPACITY), val: None }),
            Arc::new(Effect { pos: Some(Coord { x: self.pos.x+1.0, y: self.pos.y+0.0 }), typ: Some(EffectType::OPACITY), val: None }),
            Arc::new(Effect { pos: Some(Coord { x: self.pos.x+0.0, y: self.pos.y+1.0 }), typ: Some(EffectType::OPACITY), val: None })
        ]);
    }

    fn act(&mut self, cause: Arc<Vec<Arc<Effect>>>) {
        let mut inputs: Vec<bool> = Vec::new();
        for it in cause.iter() {
            if let Some(val) = it.val {
                inputs.push(if val >= 0.5 { true } else { false});
            } else { inputs.push(false) }
        }
        let outputs = self.intelligence.respond(inputs);
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
                    self.tx.send((cause,Arc::new(Effect {
                        pos: Some(pos),
                        typ: Some(EffectType::OPACITY),
                        val: Some(0.15)
                    }))).unwrap()
                },
                None => {}
            };
        };
    }

    fn tick(&self) {
        // age every tick
        self.tx.send((
                Arc::new(vec![Arc::new(Effect { pos: None, typ: Some(EffectType::TICK), val: Some(1.0)})]),
                Arc::new(Effect { pos: Some(self.pos), typ: Some(EffectType::OPACITY), val: Some(-0.01)}))
            ).unwrap();

        // alive dots sense every tick
        if self.is_alive() {
            self.tx.send((self.sense(),Arc::new(Effect { pos: Some(self.pos), typ: None, val: None }))).unwrap()
        }
    }

    fn describe(&self, typ: EffectType) -> Option<f32> {
        match typ { // BUG: will panic if typ is None
            EffectType::TICK => None,
            EffectType::OPACITY => Some(self.color[3]),
            EffectType::X => Some(self.pos.x as f32),
            EffectType::Y => Some(self.pos.y as f32),
        }
    }

    fn apply_effect(&mut self, effect: (Arc<Vec<Arc<Effect>>>,Arc<Effect>)) {
        let (_causes,effect) = effect;
        match effect.typ.unwrap() { // BUG: will panic if typ is None
            EffectType::TICK => self.tick(),
            EffectType::OPACITY => {
                self.color[3] += effect.val.unwrap(); // BUG: will panic if val is None
                if self.color[3] < 0.0 { self.color[3] = 0.0 }
                if self.color[3] > 1.0 { self.color[3] = 1.0 }
            },
            EffectType::X => self.pos.x += effect.val.unwrap() as f64, // BUG: will panic if val is None
            EffectType::Y => self.pos.y += effect.val.unwrap() as f64, // BUG: will panic if val is None
        }
    }

    fn collides_with(&self, coord: Coord) -> bool { self.pos == coord }
}