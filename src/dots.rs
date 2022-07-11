use crate::action::Action;
use crate::common::coord::Coord;
use crate::common::direction::Direction;
use crate::common::dna::{self, combine, Dna};
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
    let growth_rate = Duration::from_millis(u8::MAX as u64 * 4);
    let ticker = sleep(Duration::from_millis(0));
    tokio::pin!(ticker);
    loop {
        tokio::select! {
            () = &mut ticker => {
                let mut dot = dot.lock().await;
                if let Some(_) = dot.dna {
                    dot.age += 0.005;
                    dot.energy = f32::max(0.0,f32::min(1.0,dot.energy-dot.age));
                    if dot.energy == 0.0 {
                        dot.dna = None;
                        dot.reaction_time = None;
                        dot.age = 0.0;
                    } else {
                        dot.act().await;
                    }
                } else {
                    dot.energy += 0.005;
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
        let dna = self.dna.unwrap();
        let action: Action = thread_rng().gen();
        let direction: Direction = thread_rng().gen();
        match action {
            Action::IDLE => {}
            Action::DIGEST => {
                self.tx
                    .send_async((
                        self.reach(direction, 1.0),
                        Arc::new(Effect::ENERGY(-0.1, Some(dna.digest_mask), Some(self.pos))),
                    ))
                    .await
                    .unwrap();
            }
            Action::SEED => {
                self.tx
                    .send_async((self.reach(direction, 1.0), Arc::new(Effect::SEED(dna))))
                    .await
                    .unwrap();
            }
        };
    }

    pub fn describe(&self) -> ((f32, f32), ([f32; 3], f32)) {
        let color = match self.dna {
            Some(dna) => (dna.color, self.energy),
            None => ([1.0, 1.0, 1.0], 0.5 + self.energy / 2.0),
        };
        return ((self.pos.x as f32, self.pos.y as f32), color);
    }

    pub async fn apply_effect(&mut self, effect: Arc<Effect>) {
        match *effect {
            Effect::ENERGY(eff, mask, pos) => {
                let mask = mask.unwrap_or([1.0, 1.0, 1.0]);
                let mut delta = match self.dna {
                    Some(dna) => {
                        eff * (dna.color[0] * mask[0]
                            + dna.color[1] * mask[1]
                            + dna.color[2] * mask[2])
                            / 3.0
                    }
                    None => eff,
                };

                if self.energy + delta < 0.0 {
                    delta = 0.0 - self.energy
                }
                if self.energy + delta > 1.0 {
                    delta = 1.0 - self.energy
                }

                self.energy = f32::max(0.0, f32::min(1.0, self.energy + eff));
                if let Some(pos) = pos {
                    self.tx
                        .send_async((pos, Arc::new(Effect::ENERGY(-delta, None, None))))
                        .await
                        .unwrap();
                }
            }
            Effect::SEED(other) => {
                if !self.is_alive() {
                    if let Some(mine) = self.dna {
                        self.dna = Some(combine(mine, other));
                        self.reaction_time =
                            Some(Duration::from_millis(thread_rng().gen::<u8>() as u64));
                    } else {
                        self.dna = Some(Dna::new(other.seq));
                    }
                }
            }
        }
    }

    fn is_alive(&self) -> bool {
        return self.dna.is_some() && self.reaction_time.is_some();
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
