use futures::lock::{Mutex};
use std::collections::HashMap;
use std::sync::{Arc};

use crate::common::coord::{Coord};
use crate::dots::{Dot};

pub struct Scene {
    size: Coord,
    scale: u8,
    dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<Dot>>>>>,
}

impl Scene {
    pub fn new(size: Coord, scale: u8) -> Scene {
        let dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<Dot>>>>> = Arc::new(Mutex::new(HashMap::new()));
        Scene {
            size,
            scale,
            dots,
        }
    }

    pub async fn at(&self, pos: Coord) -> Option<Arc<Mutex<Dot>>> {
        if let Some(dot) = self.dots.lock().await.get(&pos) { return Some(dot.clone()); }

        // TODO: quickly get nearby dots to do full collision check on
        //       will be needed at the point dots can move around
        // let hashmap = self.dots.lock().await;
        // let mut nearby_dots: Vec<Arc<Mutex<Dot>>> = Vec::new();
        // for mutex in dots.iter() {
        //     let dot = mutex.lock().await;
        //     if dot.collides_with(pos) { return Some(mutex.clone()); }
        // }

        None
    }

    pub async fn push_dot(&self, pos: Coord, dot: Arc<Mutex<Dot>>) {
        if 0.0 > pos.x || pos.x >= self.size.x || 0.0 > pos.y || pos.y >= self.size.y { return; }
        self.dots.lock().await
                .entry(pos)
                .or_insert(dot.clone());
    }

    pub async fn describe(&self) -> Vec<(f64,f64,f64,([f32;3],f32))> {
        let mut ret: Vec<(f64,f64,f64,([f32;3],f32))> = Vec::new();
        let mut dots = self.dots.lock().await;
        for (key,dot) in dots.clone().iter() {
            let dot = dot.lock().await;
            if dot.task_tick.is_none() { dots.remove(key); } 
            else {
                let ((x,y),(rgb,opacity)) = dot.describe();
                ret.push((x as f64 * self.scale as f64 + 0.5 * self.scale as f64,
                            y as f64 * self.scale as f64 + 0.5 * self.scale as f64,
                            0.5 * self.scale as f64,
                            (rgb,opacity)
                        ));
            }
        }
        return ret;
    }
}