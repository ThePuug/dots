use dashmap::DashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::common::coord::Coord;
use crate::dots::Cell;

pub struct Scene {
    size: Coord,
    scale: u8,
    // Sharded concurrent map: the structure is fixed after startup, so reads
    // (at / describe) take only a per-shard lock instead of one scene-wide lock.
    dots: Arc<DashMap<Coord, Arc<Cell>>>,
}

impl Scene {
    pub fn new(size: Coord, scale: u8) -> Scene {
        Scene {
            size,
            scale,
            dots: Arc::new(DashMap::new()),
        }
    }

    pub fn at(&self, pos: Coord) -> Option<Arc<Cell>> {
        self.dots.get(&pos).map(|cell| cell.value().clone())
    }

    pub fn push_dot(&self, pos: Coord, cell: Arc<Cell>) {
        if 0.0 > pos.x || pos.x >= self.size.x || 0.0 > pos.y || pos.y >= self.size.y {
            return;
        }
        self.dots.entry(pos).or_insert(cell);
    }

    pub fn describe(&self) -> Vec<(f64, f64, f64, ([f32; 3], f32))> {
        // Lock-free: read each cell's packed render snapshot and take the
        // position from the map key. No dot is locked, so rendering never
        // contends with the simulation.
        let mut ret = Vec::with_capacity(self.dots.len());
        for cell in self.dots.iter() {
            let pos = cell.key();
            let packed = cell.value().render.load(Ordering::Relaxed);
            let rgb = [
                ((packed >> 24) & 0xff) as f32 / 255.0,
                ((packed >> 16) & 0xff) as f32 / 255.0,
                ((packed >> 8) & 0xff) as f32 / 255.0,
            ];
            let alpha = (packed & 0xff) as f32 / 255.0;
            ret.push((
                pos.x * self.scale as f64 + 0.5 * self.scale as f64,
                pos.y * self.scale as f64 + 0.5 * self.scale as f64,
                0.5 * self.scale as f64,
                (rgb, alpha),
            ));
        }
        ret
    }
}
