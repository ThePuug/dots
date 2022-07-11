use bitvec::prelude::*;
use rand::prelude::*;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

pub const SIZE: usize = 2;

#[derive(Clone, Copy, Debug)]
pub struct Dna {
    pub seq: [u64; SIZE],
    pub color: [f32; 3],
    pub digest_mask: [f32; 3],
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
    let mask: [u64; SIZE] = thread_rng().gen();
    let mut seq: [u64; SIZE] = [0; SIZE];
    for (i, it) in mask.into_iter().enumerate() {
        seq[i] = (it & mine.seq[i]) | (!it & other.seq[i]);
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
