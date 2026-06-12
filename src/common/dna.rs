use bitvec::prelude::*;
use rand::prelude::*;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::time::Duration;

pub const SIZE: usize = 23;

/// Bits of `seq` consumed by `Dna::new` for the heritable traits — colour
/// (3×6), digest_mask (3×8), reaction_time (8), seed_invest (8). The neural-net
/// weights occupy the bits after this — see `common::brain`, which decodes them
/// from the same `seq`.
pub const TRAIT_BITS: usize = 58;

/// Floor on a dot's reaction_time so metabolism can't evolve down to a 0ms
/// busy loop. The 8-bit gene adds 0..256ms on top, giving a [16, 271]ms range.
pub const REACTION_FLOOR_MS: u64 = 16;

/// Maximum energy a dot will invest in one seed; the 8-bit gene scales [0, this].
/// The cost is conserved (it becomes the offspring's starting energy), so this
/// only bounds the evolvable range — like every other trait's decode.
pub const SEED_INVEST_MAX: f32 = 0.25;

#[derive(Clone, Copy, Debug)]
pub struct Dna {
    pub seq: [u64; SIZE],
    /// Phenotype: how the dot presents to others. This is what neighbours sense
    /// and what predation targets — decoded from just the first 18 bits.
    pub color: [f32; 3],
    pub digest_mask: [f32; 3],
    pub reaction_time: Duration,
    /// Energy this dot invests into each seed (transferred to the offspring).
    pub seed_invest: f32,
    /// Identity: a colour derived from the *whole* genome so genetically similar
    /// dots look alike on screen. Shown to the viewer; not sensed by other dots.
    pub display_color: [f32; 3],
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
            seed_invest: s.f(8) * SEED_INVEST_MAX,
            display_color: genome_color(&seq),
        };
    }
}

// A locality-preserving projection of the whole genome to RGB: each channel is
// the Hamming distance from the genome to a fixed pseudo-random anchor, squashed
// through a sigmoid. Small genetic change -> small Hamming change -> small colour
// change, so a lineage holds a colour region that drifts as the genome drifts —
// unlike a hash, which would scatter near-identical genomes to random colours.
fn genome_color(seq: &[u64; SIZE]) -> [f32; 3] {
    let n = (SIZE * 64) as f32;
    let scale = (n * 0.25).sqrt(); // ~1 std of Hamming between random genomes
    let mut out = [0.5f32; 3];
    for (c, slot) in out.iter_mut().enumerate() {
        let mut hamming = 0u32;
        for (w, &word) in seq.iter().enumerate() {
            hamming += (word ^ anchor(c, w)).count_ones();
        }
        let x = (hamming as f32 - n / 2.0) / scale;
        *slot = 1.0 / (1.0 + (-x).exp());
    }
    out
}

// Deterministic pseudo-random reference word for a (channel, word) pair
// (splitmix64-style mixing) — the fixed anchors the projection measures against.
fn anchor(c: usize, w: usize) -> u64 {
    let mut x = (c as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (w as u64 + 1).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
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

    #[test]
    fn seed_invest_is_inherited_and_bounded() {
        let seq = [0x0f1e_2d3c_4b5a_6978_u64; SIZE];
        assert_eq!(Dna::new(seq).seed_invest, Dna::new(seq).seed_invest);
        let inv = Dna::new(seq).seed_invest;
        assert!(inv >= 0.0 && inv <= SEED_INVEST_MAX);
    }
}
