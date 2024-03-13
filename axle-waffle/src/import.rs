use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use anyhow::Context;
use id_arena::Id;
use indexmap::IndexMap;
use waffle::{
    entity::EntityRef, Block, Func, FuncDecl, FunctionBody, Operator, SignatureData, Value,
};

use axle::{Call, ParamID, Pick, Target, ValueID};

use crate::export::WasmSig;

pub struct ToAxle<A, B, O, T, D, S> {
    pub waffle: A,
    pub axle: B,
    pub fcache: BTreeMap<Func, FunctionBody>,
    pub fcache2: BTreeMap<(Func, Block), Id<axle::Func<O, T, D, S>>>,
}
pub enum WaffleTerm<O, T, D, S> {
    Br(Target<O, T, D, S>),
    BrIf {
        cond: ValueID,
        then: Target<O, T, D, S>,
        els: Target<O, T, D, S>,
    },
    Return(Vec<ValueID>),
    None,
}

pub fn axlize(a: Value) -> ValueID {
    return ValueID(format!("{a}"));
}
impl<
        A: Deref<Target = waffle::Module<'static>>,
        B: Deref<Target = axle::Module<O, T, D, S>> + DerefMut,
        O: From<Operator> + From<Call<O, T, D, S>> + From<Pick> + Clone,
        T: From<WaffleTerm<O, T, D, S>> + Clone,
        D,
        S: From<WasmSig> + Clone,
    > ToAxle<A, B, O, T, D, S>
{
    pub fn cache(&mut self, f: Func) -> Option<FunctionBody> {
        loop {
            if let Some(k) = self.fcache.get(&f) {
                return Some(k.clone());
            }
            let Some(b) = self.waffle.funcs[f].body() else {
                return None;
            };
            let mut c = b.clone();
            more_waffle::passes::unswitch::go(&mut c);
            c.convert_to_max_ssa(None);
            self.fcache.insert(f, c);
        }
    }
    pub fn translate(
        &mut self,
        f: Func,
        mut k: Block,
    ) -> anyhow::Result<Id<axle::Func<O, T, D, S>>> {
        if !k.is_valid() {
            let mut fb = self.cache(f).context("getting body")?;
            k = fb.entry;
        }
        loop {
            if let Some(x) = self.fcache2.get(&(f, k)) {
                return Ok(*x);
            }
            let mut fb = self.cache(f).context("getting body")?;
            let mut n = axle::Func {
                values: IndexMap::new(),
                terminator: WaffleTerm::None.into(),
                sig: WasmSig {
                    returns: fb.rets.clone(),
                    params: fb.blocks[k]
                        .params
                        .iter()
                        .enumerate()
                        .map(|a| (ParamID(a.0.to_string()), a.1 .0))
                        .collect(),
                }
                .into(),
            };
            let a = self.axle.funcs.alloc(n.clone());
            for (i, p) in fb.blocks[k].params.iter().map(|a| a.1).enumerate() {
                n.values
                    .insert(axlize(p), axle::Value::Param(ParamID(i.to_string())));
            }
            for v in fb.blocks[k].insts.clone() {
                let d = fb.values[v].clone();
                n.values.insert(
                    axlize(v),
                    match d {
                        waffle::ValueDef::BlockParam(_, _, _) => todo!(),
                        waffle::ValueDef::Operator(Operator::Call { function_index }, b, _) => {
                            let mut c = vec![];
                            for v in &fb.arg_pool[b] {
                                c.push(axlize(*v))
                            }
                            axle::Value::Operator(
                                Call {
                                    func: self.translate(function_index, Block::invalid())?,
                                }
                                .into(),
                                c,
                                vec![],
                            )
                        }
                        waffle::ValueDef::Operator(a, b, _) => {
                            let mut c = vec![];
                            for v in &fb.arg_pool[b] {
                                c.push(axlize(*v))
                            }
                            axle::Value::Operator(a.into(), c, vec![])
                        }
                        waffle::ValueDef::PickOutput(v, w, _) => axle::Value::Operator(
                            Pick { index: w as usize }.into(),
                            vec![axlize(v)],
                            vec![],
                        ),
                        waffle::ValueDef::Alias(w) => axle::Value::Alias(axlize(w)),
                        waffle::ValueDef::Placeholder(_) => todo!(),
                        waffle::ValueDef::Trace(_, _) => todo!(),
                        waffle::ValueDef::None => {
                            axle::Value::Operator(Operator::Nop.into(), vec![], vec![])
                        }
                    },
                );
            }
            n.terminator = match fb.blocks[k].terminator.clone() {
                waffle::Terminator::Br { target } => WaffleTerm::Br(Target {
                    id: self.translate(f, target.block)?,
                    args: target
                        .args
                        .iter()
                        .enumerate()
                        .map(|(n, v)| (ParamID(n.to_string()), axlize(*v)))
                        .collect(),
                })
                .into(),
                waffle::Terminator::CondBr {
                    cond,
                    if_true,
                    if_false,
                } => WaffleTerm::BrIf {
                    cond: axlize(cond),
                    then: Target {
                        id: self.translate(f, if_true.block)?,
                        args: if_true
                            .args
                            .iter()
                            .enumerate()
                            .map(|(n, v)| (ParamID(n.to_string()), axlize(*v)))
                            .collect(),
                    },
                    els: Target {
                        id: self.translate(f, if_false.block)?,
                        args: if_false
                            .args
                            .iter()
                            .enumerate()
                            .map(|(n, v)| (ParamID(n.to_string()), axlize(*v)))
                            .collect(),
                    },
                }
                .into(),
                waffle::Terminator::Select {
                    value,
                    targets,
                    default,
                } => todo!(),
                waffle::Terminator::Return { values } => {
                    WaffleTerm::Return(values.iter().map(|a| axlize(*a)).collect()).into()
                }
                waffle::Terminator::ReturnCall { func, args } => WaffleTerm::Br(Target {
                    id: self.translate(func, Block::invalid())?,
                    args: args
                        .iter()
                        .enumerate()
                        .map(|(n, v)| (ParamID(n.to_string()), axlize(*v)))
                        .collect(),
                })
                .into(),
                waffle::Terminator::ReturnCallIndirect { sig, table, args } => todo!(),
                waffle::Terminator::Unreachable => WaffleTerm::None.into(),
                waffle::Terminator::None => WaffleTerm::None.into(),
            };
            self.axle.funcs[a] = n;
            self.fcache2.insert((f, k), a);
        }
    }
}
