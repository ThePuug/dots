use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied,Vacant};
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::{Sender,Receiver};
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;

use common::coord::Coord;
use dots::Dot;
use effects::{Effect,EffectType};

pub struct Physics {
    pub tx: Sender<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>,
    pub dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<Dot>>>>>
}

impl Physics {
    pub fn new(dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<Dot>>>>>) -> Physics {
        let (tx, rx) = mpsc::channel();
        let (tx_reaper,rx_reaper) = mpsc::channel();

        let physics = Physics {
            tx: tx.clone(),
            dots: dots.clone()
        };

        // create the origin of life
        match tx.send((Arc::new(Vec::new()),Arc::new(Effect {
            pos: Some(Coord{x: 25.0, y:25.0}),
            typ: Some(EffectType::OPACITY),
            val: Some(1.0)
        }))) {
            Ok(_) => {},
            Err(msg) => println!("{}",msg)
        }

        thread::spawn(move || Physics::start_reaper(rx_reaper));
        thread::spawn(move || Physics::start(rx, tx.clone(), tx_reaper.clone(), dots.clone()));

        return physics;
    }

    pub fn start_reaper(rx_reaper: Receiver<JoinHandle<()>>) {
        for msg in rx_reaper {
            match msg.join() {
                Ok(_) => {},
                Err(_) => println!("Failed to join")
            }
        }
    }

    pub fn start(rx: Receiver<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>,
                 tx: Sender<(Arc<Vec<Arc<Effect>>>,Arc<Effect>)>, 
                 tx_reaper: Sender<JoinHandle<()>>, 
                 dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<Dot>>>>>) {
        for msg in rx {
            let (causes,effect) = msg;

            match effect.pos {
                // when the pos is not defined, then send the effect to all dots and continue
                None => {
                    let dots = dots.clone();
                    for (_,dot) in dots.lock().unwrap().iter() {
                        let dot = dot.clone();
                        let tx = tx.clone();
                        let effect = effect.clone();
                        match tx_reaper.send(thread::spawn(move || {
                            let mut dot = dot.lock().unwrap();
                            dot.update(tx, effect);
                        })) {
                            Ok(_) => {},
                            Err(msg) => println!("{}",msg)
                        }
                    }
                    continue;
                }

                // if the pos is defined ensure the dot exists
                Some(pos) => {
                    let dots = dots.clone();
                    let mut dots = dots.lock().unwrap();
                    let mut dot: Arc<Mutex<Dot>>;
                    match dots.entry(pos) {
                        Occupied(e) => dot = e.into_mut().clone(),
                        Vacant(e) => dot = e.insert(Arc::new(Mutex::new(Dot::new(pos,[0.0,0.0,0.0,0.0])))).clone()
                    }

                    // when the effect is defined, we want to update
                    match effect.typ {
                        Some(_) => {
                            let tx = tx.clone();
                            match tx_reaper.send(thread::spawn(move || {
                                let mut dot = dot.lock().unwrap();
                                dot.update(tx, effect);
                            })) {
                                Ok(_) => {},
                                Err(msg) => println!("{}", msg)
                            }
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
                            let tx = tx.clone();
                            match tx_reaper.send(thread::spawn(move || {
                                let dot = dot.lock().unwrap();
                                dot.act(Arc::new(ret_causes), tx);
                            })) {
                                Ok(_) => {},
                                Err(msg) => println!("{}", msg)
                            }
                        }
                    }
                },
            }
        }
    }
}