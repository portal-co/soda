use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};
pub mod builder;
pub mod transform;
pub mod rewrite;
use anyhow::Context;
use id_arena::*;
use indexmap::*;
#[derive(Clone,Default,Hash,Eq,PartialEq,PartialOrd, Ord)]
pub struct ValueID(pub String);
#[derive(Clone,Default,Hash,Eq,PartialEq,PartialOrd, Ord)]
pub struct ParamID(pub String);


//pub mod waffle;
pub enum Value<O, D> {
    Operator(O, Vec<ValueID>, Vec<Id<D>>),
    Param(ParamID),
    Alias(ValueID),
}
pub trait Typed<T,D>{
    fn r#typeof(&self, args: &[T], datas: &[Id<D>]) -> anyhow::Result<T>;
}
impl<O: Clone, D> Clone for Value<O, D> {
    fn clone(&self) -> Self {
        match self {
            Self::Operator(arg0, arg1, arg2) => {
                Self::Operator(arg0.clone(), arg1.clone(), arg2.clone())
            }
            Self::Param(arg0) => Self::Param(arg0.clone()),
            Self::Alias(a) => Self::Alias(a.clone()),
        }
    }
}
pub struct Call<O, T, D, S> {
    pub func: Id<crate::Func<O, T, D, S>>,
}
pub struct Pick {
    pub index: usize,
}
pub struct Func<O, T, D, S> {
    pub values: IndexMap<ValueID, Value<O, D>>,
    pub terminator: T,
    pub sig: S,
}
pub trait SigTypes<Y>{
    fn types(&self) -> anyhow::Result<BTreeMap<ParamID,Y>>;
}
impl<O,T,D,S> Func<O, T, D, S> {
    pub fn params(&self) -> IndexSet<ParamID>{
        let mut s = IndexSet::new();
        for v in self.values.values(){
            let Value::Param(p) = v else{
                continue;
            };
            s.insert(p.clone());
        }
        return s;
    }
}
impl<O,T: Default,D,S: Clone> Func<O,T,D,S>{
    pub fn bud(&self) -> TargetData<O,T,D,S>{
        let mut f = Func{
            values: IndexMap::new(),
            terminator: T::default(),
            sig: self.sig.clone()
        };
        let mut m = BTreeMap::new();
        for (i,_) in self.values.iter(){
            let t = ParamID(format!("param${}",i.0));
            m.insert(t.clone(), i.clone());
            f.values.insert(i.clone(), Value::Param(t));
        }
        return TargetData{
            func: f,
            args: m
        }; 
    }
}
pub fn types<O: Typed<Y,D>,T,D,S: SigTypes<Y>,Y: Clone>(module: &Module<O,T,D,S>, func: Id<Func<O,T,D,S>>) -> anyhow::Result<BTreeMap<ValueID,Y>>{
    let func = &module.funcs[func];
    let mut m: BTreeMap<ValueID, Y> = BTreeMap::new();
    let p = func.sig.types()?;
    for (k,v) in func.values.iter(){
        m.insert(k.clone(), match v{
            Value::Operator(o, k, d) => {
                let mut l = vec![];
                for k in k{
                    l.push(m.get(k).context("in getting a value")?.clone())
                };
                o.r#typeof(&l, d)?
            },
            Value::Param(n) => p.get(n).context("in getting a param")?.clone(),
            Value::Alias(n) => m.get(n).context("in getting a value")?.clone(),
        });
    }
    return Ok(m);
}
impl<O, T: Default, D, S: Default> Default for Func<O, T, D, S> {
    fn default() -> Self {
        Self {
            values: Default::default(),
            terminator: Default::default(),
            sig: Default::default(),
        }
    }
}
impl<O: Clone, T: Clone, D, S: Clone> Clone for Func<O, T, D, S> {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            terminator: self.terminator.clone(),
            sig: self.sig.clone(),
        }
    }
}
pub struct Module<O, T, D, S> {
    pub funcs: Arena<Func<O, T, D, S>>,
    pub data: Arena<D>,
}
pub struct Target<O, T, D, S> {
    pub id: Id<crate::Func<O, T, D, S>>,
    pub args: BTreeMap<ParamID,ValueID>,
}
pub struct TargetData<O, T, D, S> {
    pub func: crate::Func<O, T, D, S>,
    pub args: BTreeMap<ParamID,ValueID>,
}
impl<O,T,D,S> Module<O,T,D,S>{
    pub fn target_from_data(&mut self, d: TargetData<O,T,D,S>) -> Target<O,T,D,S>{
        return Target{
            id: self.funcs.alloc(d.func),
            args: d.args
        };
    }
}
// pub trait Transform<O, T, D, S> {
//     type Variant<G, A, M, E>: Transform<G, A, M, E>;
//     fn transform<G, A, M, E, C: Ctx<O, T, D, S, G, A, M, E>>(
//         &self,
//         ctx: &mut C,
//         func: impl FnMut(
//             &mut C,
//             Id<crate::Func<O, T, D, S>>,
//         ) -> anyhow::Result<Id<crate::Func<G, A, M, E>>>,
//         datum: impl FnMut(&mut C, Id<D>) -> anyhow::Result<Id<M>>,
//     ) -> anyhow::Result<Self::Variant<G, A, M, E>>;
// }

#[cfg(test)]
mod tests {
    use super::*;
}
