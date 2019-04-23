use std::collections::HashMap;
use std::collections::hash_map::Entry;
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
    pub tx: Sender<Effect>,
    pub dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<Dot>>>>>,
    tx_reaper: Sender<JoinHandle<()>>,
    hdl_reaper: JoinHandle<()>,
    hdl_rx: JoinHandle<()>
}

impl Physics {
    pub fn new() -> Physics {
        let (tx, rx) = mpsc::channel();
        let (tx_reaper,rx_reaper) = mpsc::channel();
        let dots = Arc::new(Mutex::new(HashMap::new()));

        let tx_mv = tx.clone();
        let tx_reaper_mv = tx_reaper.clone();
        let dots_mv = dots.clone();

        // create the origin of life
        tx.send(Effect {
            pos: Some(Coord{x: 25.0, y:25.0}),
            effect: EffectType::OPACITY,
            intensity: 1.0
        });

        return Physics {
            tx,
            dots,
            tx_reaper,
            hdl_reaper: thread::spawn(move || Physics::start_reaper(rx_reaper)),
            hdl_rx: thread::spawn(move || Physics::start(rx, tx_mv, tx_reaper_mv, dots_mv))
        };
    }

    pub fn start_reaper(rx_reaper: Receiver<JoinHandle<()>>) {
        for msg in rx_reaper {
            msg.join();
        }
    }

    pub fn start(rx: Receiver<Effect>, tx: Sender<Effect>, tx_reaper: Sender<JoinHandle<()>>, dots: Arc<Mutex<HashMap<Coord,Arc<Mutex<Dot>>>>>) {
        for msg in rx {
//            println!("do TICK");
            match msg.pos {
                Some(pos) => {
                    let tx = tx.clone();
                    let dots = dots.clone();
                    match dots.lock().unwrap().entry(pos) {
                        Entry::Occupied(e) => {
                            let dot = e.into_mut().clone();
                            tx_reaper.send(thread::spawn(move || {
                                let mut dot = dot.lock().unwrap();
                                dot.update(tx, msg.effect, msg.intensity);
                            }));
                        },
                        Entry::Vacant(e) => {
                            let dot = e.insert(Arc::new(Mutex::new(Dot {pos,color:[0.0,0.0,0.0,0.0]}))).clone();
                            tx_reaper.send(thread::spawn(move || {
                                let mut dot = dot.lock().unwrap();
                                dot.update(tx, msg.effect, msg.intensity);
                            }));
                        }
                    };
                }
                None => {
                    let dots = dots.clone();
                    let msg = Arc::new(msg);
                    for (_,dot) in dots.lock().unwrap().iter() {
                        let dot = dot.clone();
                        let tx = tx.clone();
                        let msg = msg.clone();
                        tx_reaper.send(thread::spawn(move || {
                            let mut dot = dot.lock().unwrap();
                            dot.update(tx, msg.effect, msg.intensity);
                        }));
                    }
                }
            }
        }
    }
}