use bitvec::prelude::*;

use crate::common::dna::{SIZE, TRAIT_BITS};

// Net shape. Inputs: 8 neighbours × (r, g, b) + own energy + one constant bias
// unit. Outputs: 8 DIGEST directions + 8 SEED directions + 1 IDLE.
pub const N_IN: usize = 8 * 3 + 2;
pub const N_HID: usize = 8;
pub const N_OUT: usize = 8 + 8 + 1;
const W_BITS: usize = 4;

/// A dot's brain: a small MLP whose weights are decoded from its DNA. It maps
/// the raw senses of the 8 surrounding cells to a score for every
/// (action, direction); the dot acts on the argmax. The structure is fixed —
/// only the weights are heritable, so behaviour evolves with the genome.
pub struct Brain {
    w1: [[f32; N_IN]; N_HID],
    w2: [[f32; N_HID]; N_OUT],
}

// Read W_BITS from the genome and map [0, 2^W_BITS) → [-1, 1). The DNA
// Sequencer only yields unsigned [0,1), so weights need their own signed map.
fn read_weight(bits: &BitSlice<u64, Lsb0>, cursor: &mut usize) -> f32 {
    let v: u32 = bits[*cursor..*cursor + W_BITS].load::<u32>();
    *cursor += W_BITS;
    (v as f32 / (1u32 << W_BITS) as f32) * 2.0 - 1.0
}

impl Brain {
    pub fn from_seq(seq: &[u64; SIZE]) -> Brain {
        let bits = seq.view_bits::<Lsb0>();
        let mut cursor = TRAIT_BITS;
        let mut w1 = [[0.0f32; N_IN]; N_HID];
        for row in w1.iter_mut() {
            for w in row.iter_mut() {
                *w = read_weight(bits, &mut cursor);
            }
        }
        let mut w2 = [[0.0f32; N_HID]; N_OUT];
        for row in w2.iter_mut() {
            for w in row.iter_mut() {
                *w = read_weight(bits, &mut cursor);
            }
        }
        Brain { w1, w2 }
    }

    pub fn forward(&self, input: &[f32; N_IN]) -> [f32; N_OUT] {
        let mut hidden = [0.0f32; N_HID];
        for (h, row) in self.w1.iter().enumerate() {
            let mut sum = 0.0;
            for (i, w) in row.iter().enumerate() {
                sum += w * input[i];
            }
            hidden[h] = sum.tanh();
        }
        let mut out = [0.0f32; N_OUT];
        for (o, row) in self.w2.iter().enumerate() {
            let mut sum = 0.0;
            for (h, w) in row.iter().enumerate() {
                sum += w * hidden[h];
            }
            out[o] = sum;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_seq_is_deterministic_and_bounded() {
        let seq = [0x0123_4567_89ab_cdef_u64; SIZE];
        let a = Brain::from_seq(&seq);
        let b = Brain::from_seq(&seq);
        assert!(a.w1.iter().zip(b.w1.iter()).all(|(x, y)| x == y));
        assert!(a.w2.iter().zip(b.w2.iter()).all(|(x, y)| x == y));
        for row in &a.w1 {
            for w in row {
                assert!(w.is_finite() && *w >= -1.0 && *w < 1.0);
            }
        }
    }

    #[test]
    fn forward_is_finite() {
        let out = Brain::from_seq(&[0xdead_beef_0bad_f00d_u64; SIZE]).forward(&[0.5; N_IN]);
        assert_eq!(out.len(), N_OUT);
        assert!(out.iter().all(|o| o.is_finite()));
    }
}
