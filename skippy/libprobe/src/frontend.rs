use std::collections::BTreeMap;

use anyhow::Context;
use axle::{Func, Module, ValueID};
use id_arena::Id;

use crate::Bytecode;
#[derive(Default)]
pub struct ValueStack{
    pub stack: Vec<ValueID>
}

pub struct Translator<O, T, D, S> {
    pub funcs: BTreeMap<Id<Func<O, T, D, S>>, Vec<Bytecode>>,
}
impl<O: Into<Bytecode> + Clone, T, D, S> Translator<O, T, D, S> {
    fn translate_func_base(
        &mut self,
        module: &Module<O, T, D, S>,
        f: Id<Func<O, T, D, S>>,
    ) -> anyhow::Result<Vec<Bytecode>> {
        if let Some(c) = self.funcs.get(&f) {
            return Ok(c.clone());
        }
        let fb = &module.funcs[f];
        let mut s = ValueStack::default();
        let mut b = vec![];
        for (s, v) in fb.values.iter() {
            match v {
                axle::Value::Operator(_, _, _) => todo!(),
                axle::Value::Param(_) => todo!(),
                axle::Value::Alias(b) => {
                }
            }
        }
        self.funcs.insert(f, b.clone());
        return Ok(b);
    }
}
