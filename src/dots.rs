use crate::action::Action;
use crate::common::coord::Coord;
use crate::common::direction::Direction;
use crate::common::dna::{self, Dna};
use crate::effect::Effect;
use flume::Sender;
use futures::lock::Mutex;
use rand::prelude::*;
use std::sync::Arc;
use tokio::task::{spawn, JoinHandle};
use tokio::time::{sleep, Duration, Instant};

pub struct DotFactory {
    tx: Sender<(Coord, Arc<Effect>)>,
}

impl DotFactory {
    pub fn new(tx: Sender<(Coord, Arc<Effect>)>) -> DotFactory {
        DotFactory { tx }
    }

    pub async fn create(
        &self,
        pos: Coord,
        seq: Option<[u64; dna::SIZE]>,
        energy: f32,
    ) -> Arc<Mutex<Dot>> {
        let dna = match seq {
            Some(seq) => Some(Dna::new(seq)),
            None => None,
        };
        let dot = Arc::new(Mutex::new(Dot::new(pos, dna, energy, self.tx.clone())));

        let ret = dot.clone();
        let ptr = dot.clone();
        let mut dot = dot.lock().await;
        dot.task_tick = Some(spawn(async move {
            ticker(ptr).await;
        }));
        return ret;
    }
}

async fn ticker(dot: Arc<Mutex<Dot>>) {
    let growth_rate = Duration::from_millis(1000);
    let ticker = sleep(Duration::from_millis(u8::MAX as u64));
    tokio::pin!(ticker);
    loop {
        tokio::select! {
            () = &mut ticker => {
                let mut dot = dot.lock().await;
                if dot.is_alive() {
                    dot.age += 0.005;
                    dot.energy = f32::max(0.0,f32::min(1.0,dot.energy-dot.age));
                    if dot.energy == 0.0 {
                        dot.dna = None;
                        dot.reaction_time = None;
                        dot.age = 0.0;
                    } else {
                        if dot.reaction_time.is_none() { dot.reaction_time = Some(Duration::from_millis(thread_rng().gen::<u8>() as u64))}
                        dot.act().await;
                    }
                } else {
                    dot.energy += 0.1;
                }

                ticker.as_mut().reset(Instant::now() + dot.reaction_time.unwrap_or(growth_rate));
            }
        }
    }
}

pub struct Dot {
    pub pos: Coord,
    pub dna: Option<Dna>,
    energy: f32,
    age: f32,
    reaction_time: Option<Duration>,
    tx: Sender<(Coord, Arc<Effect>)>,
    pub task_tick: Option<JoinHandle<()>>,
}

impl Dot {
    pub fn new(pos: Coord, dna: Option<Dna>, energy: f32, tx: Sender<(Coord, Arc<Effect>)>) -> Dot {
        let dot = Dot {
            pos,
            dna,
            energy,
            age: 0.0,
            reaction_time: None,
            tx,
            task_tick: None,
        };
        return dot;
    }

    pub async fn act(&mut self) {
        let action: Action = thread_rng().gen();
        let direction: Direction = thread_rng().gen();
        match action {
            Action::IDLE => {}
            Action::CONSUME => {
                let efficacy = thread_rng().gen::<f32>() / 5.0 / f32::MAX;
                self.energy += efficacy;
                self.tx
                    .send_async((
                        self.reach(direction, 1.0),
                        Arc::new(Effect::ENERGY((-efficacy * u16::MAX as f32) as u16)),
                    ))
                    .await
                    .unwrap();
            }
            Action::SEED => {
                self.tx
                    .send_async((
                        self.reach(direction, 1.0),
                        Arc::new(Effect::SEED(self.dna.unwrap())),
                    ))
                    .await
                    .unwrap();
            }
        };
    }

    pub fn describe(&self) -> ((f32, f32), ([f32; 3], f32)) {
        let color = match self.dna {
            Some(dna) => (dna.color, 1.0),
            None => ([1.0, 1.0, 1.0], 1.0),
        };
        return ((self.pos.x as f32, self.pos.y as f32), color);
    }

    pub fn apply_effect(&mut self, effect: Arc<Effect>) {
        match *effect {
            Effect::ENERGY(energy) => {
                self.energy = f32::max(
                    0.0,
                    f32::min(1.0, self.energy + energy as f32 / u16::MAX as f32),
                );
            }
            Effect::SEED(dna) => {
                if self.dna.is_some() {
                    self.dna.unwrap().combine(self.dna.unwrap());
                } else if !self.is_alive() {
                    self.dna = Some(Dna::new(dna.seq));
                }
            }
        }
    }

    fn is_alive(&self) -> bool {
        return self.dna.is_some();
    }

    fn reach(&self, direction: Direction, distance: f64) -> Coord {
        let sq_dist = distance * 1.41421356f64;
        return match direction {
            Direction::NORTH => Coord {
                x: self.pos.x,
                y: self.pos.y - distance,
            },
            Direction::NORTHEAST => Coord {
                x: self.pos.x + sq_dist,
                y: self.pos.y - sq_dist,
            },
            Direction::EAST => Coord {
                x: self.pos.x + distance,
                y: self.pos.y,
            },
            Direction::SOUTHEAST => Coord {
                x: self.pos.x + sq_dist,
                y: self.pos.y + sq_dist,
            },
            Direction::SOUTH => Coord {
                x: self.pos.x,
                y: self.pos.y + distance,
            },
            Direction::SOUTHWEST => Coord {
                x: self.pos.x - sq_dist,
                y: self.pos.y + sq_dist,
            },
            Direction::WEST => Coord {
                x: self.pos.x - distance,
                y: self.pos.y,
            },
            Direction::NORTHWEST => Coord {
                x: self.pos.x - sq_dist,
                y: self.pos.y - sq_dist,
            },
        };
    }
}
