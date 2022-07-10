use std::{
    hash::{Hash, Hasher},
    rc::Rc,
};

use bitvec::prelude::*;
use rand::prelude::*;

pub const SIZE: usize = 2;

#[derive(Clone, Copy, Debug)]
pub struct Dna {
    pub seq: [u64; SIZE],
    pub color: [f32; 3],
}

impl Dna {
    pub fn combine(&mut self, other: Dna) {
        let mask: [u64; 2] = thread_rng().gen();
        for (i, it) in mask.into_iter().enumerate() {
            self.seq[i] = (it & self.seq[i]) | (!it & other.seq[i]);
        }
    }

    pub fn new(seq: [u64; SIZE]) -> Dna {
        let p_seq = Rc::from(seq);
        let mut i = 0;
        return Dna {
            seq,
            color: [
                (u(p_seq.clone(), 6, {i+=000; i}) + 64) as f32 / u8::MAX as f32,
                (u(p_seq.clone(), 6, {i+=006; i}) + 64) as f32 / u8::MAX as f32,
                (u(p_seq.clone(), 6, {i+=012; i}) + 64) as f32 / u8::MAX as f32,
            ],
        };
    }
}
/*
fn f(seq: Rc<[u64; SIZE]>, n: usize, i: usize) -> f32 {
    return f32::try_from(i16::try_from(u(seq, n, i)).unwrap()).unwrap()
        / f32::try_from(2i16.pow(u32::try_from(n).unwrap())).unwrap();
}
*/
fn u(seq: Rc<[u64; SIZE]>, n: usize, i: usize) -> u32 {
    return seq.view_bits::<Lsb0>()[i..i + n].load::<u32>();
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
