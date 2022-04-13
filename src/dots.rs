use flume::{Sender};
use futures::lock::{Mutex};
use rand::prelude::*;
use std::sync::{Arc};
use tokio::task::{JoinHandle,spawn};
use tokio::time::{sleep,Duration,Instant};
use crate::common::coord::{Coord};
use crate::common::dna::{Dna};
use crate::action::{Action};
use crate::effects::{Effect,EffectType};

pub struct DotFactory {
    tx: Sender<Arc<Effect>>
}

impl DotFactory {
    pub fn new(tx: Sender<Arc<Effect>>) -> DotFactory {
        DotFactory {
            tx
        }
    }

    pub async fn create(&self, pos: Coord, seq: [u8;8], opacity: f32) -> Arc<Mutex<Dot>> {
        let dna = Dna::new(seq);

        let dot = Arc::new(Mutex::new(Dot::new(
            pos,
            dna,
            opacity,
            self.tx.clone())));
        let ret = dot.clone();
        let ptr = dot.clone();
        let mut dot = dot.lock().await;
        dot.task_tick = Some(spawn(async move { ticker(ptr).await; }));
        return ret;
    }
}

async fn idler(dot: Arc<Mutex<Dot>>) {
    let idle_duration = Duration::from_millis(250u64);
    let idler = sleep(idle_duration);
    tokio::pin!(idler);

    loop {
        tokio::select! {
            () = &mut idler => {
                let mut dot = dot.lock().await;
                if dot.is_alive() {
                    dot.act().await;
                    idler.as_mut().reset(Instant::now() + idle_duration);
                } else {
                    dot.task_idle = None;
                    break;
                }
            }
        }
    }
}

async fn ticker(dot: Arc<Mutex<Dot>>) {
    let ticker = sleep(Duration::from_millis(1000));
    tokio::pin!(ticker);
    loop {
        tokio::select! {
            () = &mut ticker => {
                let ptr = dot.clone();
                let mut dot = dot.lock().await;
                dot.age += 0.005;
                dot.opacity = f32::max(0.0,dot.opacity-dot.age);
                if dot.opacity > 0.0 {
                    if dot.task_idle.is_none() && dot.is_alive() {
                        dot.task_idle = Some(spawn(async move { idler(ptr).await; }));
                    } 
                    ticker.as_mut().reset(Instant::now() + Duration::from_millis(1000)); 
                } else { 
                    dot.task_tick = None;
                    break;
                }
            }
        }
    }
}

pub struct Dot {
    pub pos: Coord,
    pub dna: Dna,
    opacity: f32,
    age: f32,
    tx: Sender<Arc<Effect>>,
    task_idle: Option<JoinHandle<()>>,
    pub task_tick: Option<JoinHandle<()>>
}

impl Dot {
    pub fn new(pos: Coord, dna: Dna, opacity: f32, tx: Sender<Arc<Effect>>)
        -> Dot {
            let dot = Dot {
                pos,
                dna,
                opacity: opacity,
                age: 0.0,
                tx,
                task_idle: None,
                task_tick: None
            };
            return dot;
    }

    pub async fn act(&self) {
        let mut x = 0;
        let mut y = 0;
        let mut rng: StdRng = SeedableRng::from_entropy();
        let outputs: [bool; 5] = rng.gen();
        if outputs[0] { y -= 1; }
        if outputs[1] { x += 1; }
        if outputs[2] { y += 1; }
        if outputs[3] { x -= 1; }
        let action = if outputs[4] { Action::DARKEN } else { Action::IDLE };
        if action != Action::IDLE {
            match self.reach(x,y) {
                Some(pos) => {
                    self.tx.send_async(Arc::new(Effect {
                        pos: Some(pos),
                        typ: Some(EffectType::OPACITY),
                        val: Some(self.dna.seq)
                    })).await.unwrap();
                },
                None => {}
            };
        };
    }

    pub fn describe(&self) -> ((f32,f32),([f32;3],f32)) {
        return (
            (self.pos.x as f32,self.pos.y as f32),
            (self.dna.color(),self.opacity)
        );
    }

    pub fn apply_effect(&mut self, effect: Arc<Effect>) {
        match effect.typ.unwrap() {
            EffectType::OPACITY => { 
                if self.is_alive() { self.opacity = f32::max(0.0,f32::min(1.0,self.opacity + 0.1)); } 
                else {
                    self.opacity = f32::max(0.0,f32::min(1.0,self.opacity + 0.1));
                    if self.is_alive() { self.dna.combine(effect.val.unwrap()); }
                }
            },
            // EffectType::X => self.pos.x += effect.val.unwrap() as f64,
            // EffectType::Y => self.pos.y += effect.val.unwrap() as f64,
        }
    }

    // pub fn collides_with(&self, coord: Coord) -> bool { self.pos == coord }

    fn is_alive(&self) -> bool { return self.opacity >= 0.3; }

    fn reach(&self, offset_x: i8, offset_y: i8) -> Option<Coord> {
        let x = self.pos.x + offset_x as f64;
        let y = self.pos.y + offset_y as f64;
        return Some(Coord { x: x, y: y });
    }
}
