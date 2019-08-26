use std::collections::hash_map::Entry::{Occupied,Vacant};
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::{Sender,Receiver};
use std::sync::Mutex;

use common::coord::Coord;
use common::threadpool::ThreadPool;
use dots::Dot;
use effects::{Effect,EffectType};
use scene::Scene;

pub struct Physics {
    pub scene: Scene,
    rx: Receiver<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>,
    tx: Sender<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>,
    pool: ThreadPool
}

impl Physics {
    pub fn new(pool: ThreadPool,
               scene: Scene) -> Physics {
        let (tx, rx) = mpsc::channel();

        let physics = Physics {
            scene,
            rx,
            tx,
            pool
        };

        // create the origin of life
        physics.queue((Arc::new(Vec::new()),Arc::new(Effect {
            pos: Some(Coord{x: 5.0, y:5.0}),
            typ: Some(EffectType::OPACITY),
            val: Some(1.0)
        })));

        physics
    }

    pub fn queue(&self, msg: (Arc<Vec<Arc<Effect>>>,Arc<Effect>)) {
        self.tx.send(msg).unwrap();
    }

    pub fn apply(&self) {
        loop {
            let msg = self.rx.try_recv();
            if msg.is_err() { break; }
            let (causes,effect) = msg.unwrap();

            match effect.pos {
                // when the pos is not defined, then send the effect to all dots and continue
                None => {
                    let dots = self.scene.dots.clone();
                    for (_,dot) in dots.lock().unwrap().iter() {
                        let dot = dot.clone();
                        let tx = self.tx.clone();
                        let effect = effect.clone();
                        self.pool.run(move || {
                            let mut dot = dot.lock().unwrap();
                            dot.update(tx, effect);
                        });
                    }
                    continue;
                }

                // if the pos is defined ensure the dot exists
                Some(pos) => {
                    let dots = self.scene.dots.clone();
                    let mut dots = dots.lock().unwrap();
                    let mut dot: Arc<Mutex<Dot>>;
                    match dots.entry(pos) {
                        Occupied(e) => dot = e.into_mut().clone(),
                        Vacant(e) => dot = e.insert(Arc::new(Mutex::new(Dot::new(pos,[0.0,0.0,0.0,0.0])))).clone()
                    }

                    // when the effect is defined, we want to update
                    match effect.typ {
                        Some(_) => {
                            let tx = self.tx.clone();
                            self.pool.run(move || {
                                let mut dot = dot.lock().unwrap();
                                dot.update(tx, effect);
                            });
                        },

                        // when the effect is undefined, then the dot is sensing
                        None => {
                            // so populate the vals of the causes
                            let mut ret_causes: Vec<Arc<Effect>> = Vec::new();
                            for cause in causes.iter() {
                                match cause.pos {
                                    Some(pos) => {
                                        match dots.entry(pos) {
                                            Occupied(e) => {
                                                let dot = e.into_mut().clone();
                                                match cause.typ {
                                                    Some(typ) => {
                                                        match typ {
                                                            EffectType::OPACITY => ret_causes.push(Arc::new(Effect { pos: cause.pos, typ: Some(typ), val: Some(dot.lock().unwrap().color[3]) })),
                                                            _ => {}
                                                        }
                                                    },
                                                    None => {}
                                                }
                                            },
                                            Vacant(_) => {
                                                match cause.typ {
                                                    Some(typ) => {
                                                        match typ {
                                                            EffectType::OPACITY => ret_causes.push(Arc::new(Effect { pos: cause.pos, typ: Some(typ), val: Some(0.0) })),
                                                            _ => {}
                                                        }
                                                    },
                                                    None => {}
                                                }
                                            }
                                        }
                                    },
                                    None => {}
                                }
                            }

                            // and ask the dot to respond
                            let tx = self.tx.clone();
                            self.pool.run(move || {
                                let dot = dot.lock().unwrap();
                                dot.act(tx, Arc::new(ret_causes));
                            });
                        }
                    }
                },
            }
        }
    }
}