// use std::collections::HashMap;
use std::sync::{Arc,Mutex};

use common::coord::{Coord};
use dots::{IDot};
use effects::{Effect,EffectType};

pub struct Scene<TDot> {
    scale: u8,
    // dots: Arc<Mutex<HashMap<(u32,u32),Vec<Arc<Mutex<TDot>>>>>>, 
    dots: Arc<Mutex<Vec<Arc<Mutex<TDot>>>>>, //OLD
}

impl<TDot> Scene<TDot> 
    where TDot : IDot {
    pub fn new(scale: u8) -> Scene<TDot> {
        // let dots: Arc<Mutex<HashMap<(u32,u32),Vec<Arc<Mutex<TDot>>>>>> = Arc::new(Mutex::new(HashMap::new()));
        let dots: Arc<Mutex<Vec<Arc<Mutex<TDot>>>>> = Arc::new(Mutex::new(Vec::new())); //OLD
        Scene {
            scale,
            dots,
        }
    }

    pub fn at(&self, pos: Coord) -> Option<Arc<Mutex<TDot>>> {
        // let mut hashmap = self.dots.lock().unwrap();
        // let mut dots: Vec<Arc<Mutex<TDot>>> = Vec::new();
        // if let Some(it) = hashmap.get_mut(&(pos.x as u32 - 1, pos.y as u32 - 1)) { dots.append(it); }
        // if let Some(it) = hashmap.get_mut(&(pos.x as u32 - 1, pos.y as u32 + 0)) { dots.append(it); }
        // if let Some(it) = hashmap.get_mut(&(pos.x as u32 - 1, pos.y as u32 + 1)) { dots.append(it); }
        // if let Some(it) = hashmap.get_mut(&(pos.x as u32 + 0, pos.y as u32 - 1)) { dots.append(it); }
        // if let Some(it) = hashmap.get_mut(&(pos.x as u32 + 0, pos.y as u32 + 0)) { dots.append(it); }
        // if let Some(it) = hashmap.get_mut(&(pos.x as u32 + 0, pos.y as u32 + 1)) { dots.append(it); }
        // if let Some(it) = hashmap.get_mut(&(pos.x as u32 + 1, pos.y as u32 - 1)) { dots.append(it); }
        // if let Some(it) = hashmap.get_mut(&(pos.x as u32 + 1, pos.y as u32 + 0)) { dots.append(it); }
        // if let Some(it) = hashmap.get_mut(&(pos.x as u32 + 1, pos.y as u32 + 1)) { dots.append(it); }
        let dots = self.dots.lock().unwrap(); //OLD
        for mutex in dots.iter() {
            let dot = mutex.lock().unwrap();
            if dot.collides_with(pos) { return Some(mutex.clone()); }
        }
        None
    }

    pub fn push_dot(&self, dot: TDot) -> Arc<Mutex<TDot>> {
        let new = Arc::new(Mutex::new(dot));
        // let dot = Arc::new(new.lock().unwrap());
        self.dots.lock().unwrap()
                // .entry((dot.describe(EffectType::X).unwrap() as u32, dot.describe(EffectType::Y).unwrap() as u32))
                // .or_insert(Vec::new())
            .push(new.clone());
        new.clone() // TODO: why do I need clone()?
    }

    pub fn apply(&self, effect: (Arc<Vec<Arc<Effect>>>,Arc<Effect>)) {
        let (_causes,effect) = effect;
        // for vec in self.dots.lock().unwrap().values() {            
            // for mutex in vec.iter() {
            for mutex in self.dots.lock().unwrap().iter() { //OLD
                let x: f64;
                let y: f64;
                { // TODO: smelly code, two locks
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
        // }
    }

    pub fn describe(&self) -> Vec<(f64,f64,f64,f32)> {
        let mut ret: Vec<(f64,f64,f64,f32)> = Vec::new();
        // for vec in self.dots.lock().unwrap().values() {
            // for mutex in vec.iter() {
            for mutex in self.dots.lock().unwrap().iter() { //OLD
                let dot = mutex.lock().unwrap();
                ret.push((dot.describe(EffectType::X).unwrap() as f64 * self.scale as f64 + 0.5 * self.scale as f64,
                          dot.describe(EffectType::Y).unwrap() as f64 * self.scale as f64 + 0.5 * self.scale as f64,
                          0.5 * self.scale as f64,
                          dot.describe(EffectType::OPACITY).unwrap()));
            }
        // }
        ret
    }
}