use std::collections::HashMap;
use std::sync::{Arc,Mutex};

use common::coord::{Coord};
use dots::{IDot};
use effects::{Effect,EffectType};

pub struct Scene<TDot> {
    scale: u8,
    dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<TDot>>>>>,
}

impl<TDot> Scene<TDot> 
    where TDot : IDot {
    pub fn new(scale: u8) -> Scene<TDot> {
        let dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<TDot>>>>> = Arc::new(Mutex::new(HashMap::new()));
        Scene {
            scale,
            dots,
        }
    }

    pub fn at(&self, pos: Coord) -> Option<Arc<Mutex<TDot>>> {
        let hashmap = self.dots.lock().unwrap();
        let mut dots: Vec<Arc<Mutex<TDot>>> = Vec::new();
        // if let Some(dot) = hashmap.get(&Coord{x:pos.x-1.0,y:pos.y-1.0}) { dots.push(dot.clone()); }
        // if let Some(dot) = hashmap.get(&Coord{x:pos.x-1.0,y:pos.y+0.0}) { dots.push(dot.clone()); }
        // if let Some(dot) = hashmap.get(&Coord{x:pos.x-1.0,y:pos.y+1.0}) { dots.push(dot.clone()); }
        // if let Some(dot) = hashmap.get(&Coord{x:pos.x+0.0,y:pos.y-1.0}) { dots.push(dot.clone()); }
        if let Some(dot) = hashmap.get(&pos) { dots.push(dot.clone()); }
        // if let Some(dot) = hashmap.get(&Coord{x:pos.x+0.0,y:pos.y+1.0}) { dots.push(dot.clone()); }
        // if let Some(dot) = hashmap.get(&Coord{x:pos.x+1.0,y:pos.y-1.0}) { dots.push(dot.clone()); }
        // if let Some(dot) = hashmap.get(&Coord{x:pos.x+1.0,y:pos.y+0.0}) { dots.push(dot.clone()); }
        // if let Some(dot) = hashmap.get(&Coord{x:pos.x+1.0,y:pos.y+1.0}) { dots.push(dot.clone()); }
        for mutex in dots.iter() {
            let dot = mutex.lock().unwrap();
            if dot.collides_with(pos) { return Some(mutex.clone()); }
        }
        None
    }

    pub fn push_dot(&self, dot: TDot) -> Arc<Mutex<TDot>> {
        let x = dot.describe(EffectType::X).unwrap() as f64;
        let y = dot.describe(EffectType::Y).unwrap() as f64;
        let dot = Arc::new(Mutex::new(dot));
        self.dots.lock().unwrap()
                .entry(Coord{ x, y })
                .or_insert(dot.clone());
        dot
    }

    pub fn apply(&self, effect: (Arc<Vec<Arc<Effect>>>,Arc<Effect>)) {
        let (_causes,effect) = effect;
        for mutex in self.dots.lock().unwrap().values() {
            let x: f64;
            let y: f64;
            { // TODO: smelly code, two locks??
                let dot = mutex.lock().unwrap();
                x = dot.describe(EffectType::X).unwrap() as f64;
                y = dot.describe(EffectType::Y).unwrap() as f64;
            }
            let mut dot = mutex.lock().unwrap();
            dot.apply_effect((Arc::new(vec![effect.clone()]),Arc::new(Effect{
                pos: Some(Coord { x, y }), 
                typ: effect.typ,
                val: effect.val
            })));
        }
    }

    pub fn describe(&self) -> Vec<(f64,f64,f64,f32)> {
        let mut ret: Vec<(f64,f64,f64,f32)> = Vec::new();
        for mutex in self.dots.lock().unwrap().values() {
            let dot = mutex.lock().unwrap();
            ret.push((dot.describe(EffectType::X).unwrap() as f64 * self.scale as f64 + 0.5 * self.scale as f64,
                        dot.describe(EffectType::Y).unwrap() as f64 * self.scale as f64 + 0.5 * self.scale as f64,
                        0.5 * self.scale as f64,
                        dot.describe(EffectType::OPACITY).unwrap()));
        }
        ret
    }
}