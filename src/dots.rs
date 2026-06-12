use crate::action::Action;
use crate::common::coord::Coord;
use crate::common::direction::Direction;
use crate::common::brain::{Brain, N_IN, N_OUT};
use crate::common::dna::{self, combine, Dna};
use crate::effect::Effect;
use crate::scene::Scene;
use flume::Sender;
use futures::lock::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::task::{spawn, JoinHandle};
use tokio::time::{sleep, Duration, Instant};

/// A grid cell: the simulation state behind an async Mutex, plus two lock-free
/// RGBA8 snapshots refreshed after every mutation. `sense` is the phenotype
/// other dots perceive; `render` is the whole-genome colour shown to the viewer.
/// Both are read without taking the dot's lock.
pub struct Cell {
    pub sense: AtomicU32,
    pub render: AtomicU32,
    pub dot: Mutex<Dot>,
}

impl Cell {
    // Recompute both snapshots from the dot's current state.
    pub fn refresh_snapshots(&self, dot: &Dot) {
        self.sense.store(dot.pack_sense(), Ordering::Relaxed);
        self.render.store(dot.pack_render(), Ordering::Relaxed);
    }
}

pub struct DotFactory {
    tx: Sender<(Coord, Arc<Effect>)>,
    scene: Arc<Scene>,
}

impl DotFactory {
    pub fn new(tx: Sender<(Coord, Arc<Effect>)>, scene: Arc<Scene>) -> DotFactory {
        DotFactory { tx, scene }
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
            sense: AtomicU32::new(0),
            render: AtomicU32::new(0),
            dot: Mutex::new(Dot::new(pos, dna, energy, self.tx.clone())),
        });

        let ptr = cell.clone();
        let scene = self.scene.clone();
        let mut dot = cell.dot.lock().await;
        cell.refresh_snapshots(&dot);
        dot.task_tick = Some(spawn(async move {
            ticker(ptr, scene).await;
        }));
        drop(dot);
        return cell;
    }
}

async fn ticker(cell: Arc<Cell>, scene: Arc<Scene>) {
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
                        dot.refresh_brain();
                    } else {
                        let senses = dot.neighbors().map(|c| scene.sense(c));
                        dot.act(senses).await;
                    }
                } else {
                    dot.energy += 0.005;
                }

                // refresh the lock-free snapshots after mutating
                cell.refresh_snapshots(&dot);
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
    brain: Option<Brain>,
}

// Read the net's choice: the highest-scoring output selects both the action
// and its direction. No heuristics — the genome's weights alone decide
// (ties resolve to the lower index, which is vanishingly rare for f32 scores).
fn decide(out: &[f32; N_OUT]) -> Option<(Action, Direction)> {
    let mut best = 0;
    for i in 1..N_OUT {
        if out[i] > out[best] {
            best = i;
        }
    }
    if best < 8 {
        Some((Action::DIGEST, Direction::from_index(best)))
    } else if best < 16 {
        Some((Action::SEED, Direction::from_index(best - 8)))
    } else {
        None // IDLE
    }
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
            brain: dna.map(|d| Brain::from_seq(&d.seq)),
        };
        return dot;
    }

    pub async fn act(&mut self, senses: [u32; 8]) {
        let brain = match &self.brain {
            Some(brain) => brain,
            None => return,
        };

        // raw perception: each neighbour's (r, g, b), own energy, bias unit.
        let mut input = [0.0f32; N_IN];
        for (n, &s) in senses.iter().enumerate() {
            input[3 * n] = ((s >> 24) & 0xff) as f32 / 255.0;
            input[3 * n + 1] = ((s >> 16) & 0xff) as f32 / 255.0;
            input[3 * n + 2] = ((s >> 8) & 0xff) as f32 / 255.0;
        }
        input[24] = self.energy;
        input[25] = 1.0;

        let decision = decide(&brain.forward(&input));
        let dna = self.dna.unwrap();
        match decision {
            None => {} // IDLE
            Some((Action::DIGEST, direction)) => {
                self.tx
                    .send_async((
                        self.reach(direction, 1.0),
                        Arc::new(Effect::ENERGY(-0.1, Some(dna.digest_mask), Some(self.pos))),
                    ))
                    .await
                    .unwrap();
            }
            Some((Action::SEED, direction)) => {
                self.tx
                    .send_async((self.reach(direction, 1.0), Arc::new(Effect::SEED(dna))))
                    .await
                    .unwrap();
            }
            Some((Action::IDLE, _)) => {}
        }
    }

    // Re-decode the brain from the current DNA. Called whenever DNA changes so
    // the brain always matches the genome (None when the dot is dead/empty).
    fn refresh_brain(&mut self) {
        self.brain = self.dna.map(|d| Brain::from_seq(&d.seq));
    }

    // The 8 neighbour coords in net-output direction order, so senses[i] lines
    // up with Direction::from_index(i) and with where the dot will act.
    fn neighbors(&self) -> [Coord; 8] {
        let mut n = [self.pos; 8];
        for (i, c) in n.iter_mut().enumerate() {
            *c = self.reach(Direction::from_index(i), 1.0);
        }
        n
    }

    // Pack appearance into RGBA8. A live dot shows `alive_rgb` fully opaque; a
    // dead/empty cell shows white with opacity ramping from its energy. Position
    // is the map key, so it isn't packed.
    fn pack(&self, alive_rgb: [f32; 3]) -> u32 {
        let (rgb, alpha): ([f32; 3], f32) = match self.dna {
            Some(_) => (alive_rgb, 1.0),
            None => ([1.0, 1.0, 1.0], 0.5 + self.energy / 2.0),
        };
        let q = |f: f32| -> u32 { (f.clamp(0.0, 1.0) * 255.0 + 0.5) as u32 };
        (q(rgb[0]) << 24) | (q(rgb[1]) << 16) | (q(rgb[2]) << 8) | q(alpha)
    }

    /// What neighbours perceive: the phenotype colour the dot presents.
    pub fn pack_sense(&self) -> u32 {
        self.pack(self.dna.map_or([1.0; 3], |d| d.color))
    }

    /// What the viewer sees: a colour derived from the whole genome, so
    /// genetically-similar dots look alike (the phenotype is hidden from view).
    pub fn pack_render(&self) -> u32 {
        self.pack(self.dna.map_or([1.0; 3], |d| d.display_color))
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
                        // fertilise: recombine, and inherit metabolism from the child
                        let child = combine(mine, other);
                        self.reaction_time = Some(child.reaction_time);
                        self.dna = Some(child);
                    } else {
                        self.dna = Some(Dna::new(other.seq));
                    }
                    self.refresh_brain();
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

    // sense packs the phenotype (what neighbours see); render packs the
    // whole-genome colour (what the viewer sees). Dead cells are white with
    // energy-based opacity in both.
    #[test]
    fn sense_is_phenotype_render_is_genome_colour() {
        let eps = 1.0 / 255.0;
        let (tx, _rx): (Sender<(Coord, Arc<Effect>)>, _) = flume::unbounded();

        let dead = Dot::new(Coord { x: 1.0, y: 2.0 }, None, 0.4, tx.clone());
        for packed in [dead.pack_sense(), dead.pack_render()] {
            let (rgb, a) = unpack(packed);
            assert!(rgb.iter().all(|c| (c - 1.0).abs() <= eps), "dead is white");
            assert!((a - (0.5 + 0.4 / 2.0)).abs() <= eps, "dead opacity ramps with energy");
        }

        let dna = Dna::new([0x1234_5678_9abc_def0_u64; dna::SIZE]);
        let alive = Dot::new(Coord { x: 0.0, y: 0.0 }, Some(dna), 0.3, tx);

        let (rgb, a) = unpack(alive.pack_sense());
        for i in 0..3 {
            assert!((rgb[i] - dna.color[i]).abs() <= eps, "sense shows phenotype [{}]", i);
        }
        assert!((a - 1.0).abs() <= eps);

        let (rgb, a) = unpack(alive.pack_render());
        for i in 0..3 {
            assert!((rgb[i] - dna.display_color[i]).abs() <= eps, "render shows genome [{}]", i);
        }
        assert!((a - 1.0).abs() <= eps);
    }

    // decide() must map output indices to the right action+direction space:
    // 0..8 DIGEST, 8..16 SEED, 16 IDLE.
    #[test]
    fn decide_maps_output_index_to_action() {
        let mut out = [0.0f32; N_OUT];
        out[2] = 1.0;
        assert!(matches!(decide(&out), Some((Action::DIGEST, _))));

        let mut out = [0.0f32; N_OUT];
        out[10] = 1.0;
        assert!(matches!(decide(&out), Some((Action::SEED, _))));

        let mut out = [0.0f32; N_OUT];
        out[16] = 1.0;
        assert!(decide(&out).is_none());
    }
}
