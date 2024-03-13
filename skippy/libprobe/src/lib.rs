use std::{
    collections::BTreeMap,
    marker::PhantomData,
    mem::take,
    ops::{Add, AddAssign},
};

use axle::{Func, Module};
use bimap::{BiBTreeMap, BiMap};
use id_arena::Id;
pub mod rust;
pub mod frontend;
#[derive(Eq, PartialEq, PartialOrd, Ord, Clone)]
pub enum Bytecode {
    Const(u64),
    Dig(Vec<u64>),
}


pub fn peep(v: &mut Vec<Bytecode>) {
    let mut d = vec![];
    for w in take(v) {
        match w {
            Bytecode::Const(c) => {
                if d.len() != 0 {
                    v.push(Bytecode::Dig(d))
                }
                v.push(Bytecode::Const(c));
                d = vec![];
            }
            Bytecode::Dig(e) => d.extend(e.into_iter()),
        }
    }
}
#[derive(Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct Palette {
    pub palette: BiBTreeMap<Bytecode, u32>,
}
impl Palette {
    pub fn need(&mut self, a: Bytecode) -> u32 {
        if let Some(r) = self.palette.get_by_left(&a) {
            return *r;
        }
        let mut i = 0;
        while self.palette.contains_right(&i) {
            i += 1
        }
        self.palette.insert(a, i);
        return i;
    }
}
#[derive(Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct Render {
    pub palette: Palette,
    pub all: Vec<u32>,
}
#[cfg(test)]
mod tests {
    use super::*;
}
