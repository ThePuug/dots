use crate::action::Action;
use crate::common::coord::Coord;
use crate::common::direction::Direction;
use crate::common::dna::{self, combine, Dna};
use crate::effect::Effect;
use flume::Sender;
use futures::lock::Mutex;
use rand::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::task::{spawn, JoinHandle};
use tokio::time::{sleep, Duration, Instant};

/// A grid cell: the simulation state behind an async Mutex, plus a lock-free
/// packed render snapshot (RGBA8). The dot refreshes `render` after every
/// mutation so the renderer can read a cell's appearance without taking its lock.
pub struct Cell {
    pub render: AtomicU32,
    pub dot: Mutex<Dot>,
}

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
    ) -> Arc<Cell> {
        let dna = match seq {
            Some(seq) => Some(Dna::new(seq)),
            None => None,
        };
        let cell = Arc::new(Cell {
            render: AtomicU32::new(0),
            dot: Mutex::new(Dot::new(pos, dna, energy, self.tx.clone())),
        });

        let ptr = cell.clone();
        let mut dot = cell.dot.lock().await;
        cell.render.store(dot.pack_render(), Ordering::Relaxed);
        dot.task_tick = Some(spawn(async move {
            ticker(ptr).await;
        }));
        drop(dot);
        return cell;
    }
}

async fn ticker(cell: Arc<Cell>) {
    let growth_rate = Duration::from_millis(u8::MAX as u64 * 4);
    let ticker = sleep(Duration::from_millis(0));
    tokio::pin!(ticker);
    loop {
        tokio::select! {
            () = &mut ticker => {
                let mut dot = cell.dot.lock().await;
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

                // refresh the lock-free render snapshot after mutating
                cell.render.store(dot.pack_render(), Ordering::Relaxed);
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

    /// Pack the cell's current appearance into RGBA8 for the lock-free render
    /// snapshot. This folds in the rule the scene used to apply: a live dot
    /// shows its DNA colour fully opaque; a dead dot shows white with opacity
    /// ramping from its energy. Position is the map key, so it isn't packed.
    pub fn pack_render(&self) -> u32 {
        let (rgb, alpha): ([f32; 3], f32) = match self.dna {
            Some(dna) => (dna.color, 1.0),
            None => ([1.0, 1.0, 1.0], 0.5 + self.energy / 2.0),
        };
        let q = |f: f32| -> u32 { (f.clamp(0.0, 1.0) * 255.0 + 0.5) as u32 };
        (q(rgb[0]) << 24) | (q(rgb[1]) << 16) | (q(rgb[2]) << 8) | q(alpha)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::dna::Dna;

    fn unpack(p: u32) -> ([f32; 3], f32) {
        (
            [
                ((p >> 24) & 0xff) as f32 / 255.0,
                ((p >> 16) & 0xff) as f32 / 255.0,
                ((p >> 8) & 0xff) as f32 / 255.0,
            ],
            (p & 0xff) as f32 / 255.0,
        )
    }

    // pack_render must reproduce the (rgb, opacity) the old describe()+scene
    // path produced: live -> DNA colour, opaque; dead -> white, 0.5 + energy/2.
    #[test]
    fn pack_render_round_trips_to_old_appearance() {
        let eps = 1.0 / 255.0;
        let (tx, _rx): (Sender<(Coord, Arc<Effect>)>, _) = flume::unbounded();

        let dead = Dot::new(Coord { x: 1.0, y: 2.0 }, None, 0.4, tx.clone());
        let (rgb, a) = unpack(dead.pack_render());
        assert!(rgb.iter().all(|c| (c - 1.0).abs() <= eps), "dead is white");
        assert!((a - (0.5 + 0.4 / 2.0)).abs() <= eps, "dead opacity ramps with energy");

        let dna = Dna::new([0x1234_5678_9abc_def0, 0x0fed_cba9_8765_4321]);
        let alive = Dot::new(Coord { x: 0.0, y: 0.0 }, Some(dna), 0.3, tx);
        let (rgb, a) = unpack(alive.pack_render());
        for i in 0..3 {
            assert!((rgb[i] - dna.color[i]).abs() <= eps, "live shows DNA colour [{}]", i);
        }
        assert!((a - 1.0).abs() <= eps, "live is opaque");
    }
}
