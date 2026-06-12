use bitvec::prelude::*;
use rand::prelude::*;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::time::Duration;

pub const SIZE: usize = 23;

/// Bits of `seq` consumed by `Dna::new` for the heritable traits — colour
/// (3×6), digest_mask (3×8), and reaction_time (8). The neural-net weights
/// occupy the bits after this — see `common::brain`, which decodes them from
/// the same `seq`.
pub const TRAIT_BITS: usize = 50;

/// Floor on a dot's reaction_time so metabolism can't evolve down to a 0ms
/// busy loop. The 8-bit gene adds 0..256ms on top, giving a [16, 271]ms range.
pub const REACTION_FLOOR_MS: u64 = 16;

#[derive(Clone, Copy, Debug)]
pub struct Dna {
    pub seq: [u64; SIZE],
    pub color: [f32; 3],
    pub digest_mask: [f32; 3],
    pub reaction_time: Duration,
}

impl Dna {
    pub fn new(seq: [u64; SIZE]) -> Dna {
        let mut s = Sequencer {
            seq: Rc::from(seq),
            cursor: 0,
        };
        return Dna {
            seq,
            color: [
                (s.u(6) + 64) as f32 / u8::MAX as f32,
                (s.u(6) + 64) as f32 / u8::MAX as f32,
                (s.u(6) + 64) as f32 / u8::MAX as f32,
            ],
            digest_mask: [s.f(8), s.f(8), s.f(8)], //[0.0, 0.0, 0.0],
            reaction_time: Duration::from_millis(REACTION_FLOOR_MS + s.u(8) as u64),
        };
    }
}

impl PartialEq for Dna {
    fn eq(&self, other: &Self) -> bool {
        self.seq == other.seq
    }
}
impl Eq for Dna {}
impl Hash for Dna {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.seq.hash(state);
    }
}

pub fn combine(mine: Dna, other: Dna) -> Dna {
    let mut rng = thread_rng();
    let mask: [u64; SIZE] = rng.gen();
    let mut seq: [u64; SIZE] = [0; SIZE];
    for (i, it) in mask.into_iter().enumerate() {
        seq[i] = (it & mine.seq[i]) | (!it & other.seq[i]);
    }
    // mutation: flip a few random bits so the gene pool can innovate new
    // weights/colours/diets, not just reshuffle the parents' alleles.
    let bits = SIZE * 64;
    for _ in 0..rng.gen_range(0..=3) {
        let b = rng.gen_range(0..bits);
        seq[b / 64] ^= 1u64 << (b % 64);
    }
    Dna::new(seq)
}

struct Sequencer {
    cursor: usize,
    seq: Rc<[u64; SIZE]>,
}

impl Sequencer {
    fn f(&mut self, n: usize) -> f32 {
        return f32::try_from(i16::try_from(self.u(n)).unwrap()).unwrap()
            / f32::try_from(2i16.pow(u32::try_from(n).unwrap())).unwrap();
    }

    fn u(&mut self, n: usize) -> u32 {
        self.cursor += n;
        return self.seq.view_bits::<Lsb0>()[self.cursor - n..self.cursor].load::<u32>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reaction_time_is_inherited_and_floored() {
        let seq = [0xabcd_ef01_2345_6789_u64; SIZE];
        // derived from the genome, not random: same seq -> same metabolism.
        assert_eq!(Dna::new(seq).reaction_time, Dna::new(seq).reaction_time);
        let ms = Dna::new(seq).reaction_time.as_millis() as u64;
        assert!(ms >= REACTION_FLOOR_MS && ms <= REACTION_FLOOR_MS + 255);
    }
}
