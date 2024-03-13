use crate::*;

pub trait Transform<O, T, D, S, G, A, M, E, X> {
    type Result;
    fn rewrite<C: Ctx<O, T, D, S, G, A, M, E, X>>(
        &self,
        ctx: &mut C,
        func: impl FnMut(
            &mut C,
            Id<crate::Func<O, T, D, S>>,
        ) -> anyhow::Result<Id<crate::Func<G, A, M, E>>>,
        datum: impl FnMut(&mut C, Id<D>) -> anyhow::Result<Id<M>>,
    ) -> anyhow::Result<Self::Result>;
}

pub struct Transformer<B, C, O, T, D, S, G, A, M, E, X> {
    pub input: B,
    pub output: C,
    pub fcache: BTreeMap<Id<crate::Func<O, T, D, S>>, Id<crate::Func<G, A, M, E>>>,
    pub dcache: BTreeMap<Id<D>, Id<M>>,
    pub user: X,
}
pub trait Ctx<O, T, D, S, G, A, M, E, X> {
    fn input(&self) -> &Module<O, T, D, S>;
    fn output(&self) -> &Module<G, A, M, E>;
    fn output_mut(&mut self) -> &mut Module<G, A, M, E>;
    fn user(&self) -> &X;
    fn user_mut(&mut self) -> &mut X;
}
impl<
        B: Deref<Target = Module<O, T, D, S>>,
        C: Deref<Target = Module<G, A, M, E>> + DerefMut,
        O,
        T,
        D,
        S,
        G,
        A,
        M,
        E,
        X,
    > Ctx<O, T, D, S, G, A, M, E, X> for Transformer<B, C, O, T, D, S, G, A, M, E, X>
{
    fn input(&self) -> &Module<O, T, D, S> {
        return &self.input;
    }

    fn output(&self) -> &Module<G, A, M, E> {
        return &self.output;
    }

    fn output_mut(&mut self) -> &mut Module<G, A, M, E> {
        return &mut self.output;
    }

    fn user(&self) -> &X {
        return &self.user;
    }

    fn user_mut(&mut self) -> &mut X {
        return &mut self.user;
    }
}
impl<
        B: Deref<Target = Module<O, T, D, S>>,
        C: Deref<Target = Module<G, A, M, E>> + DerefMut,
        O: Transform<O, T, D, S, G, A, M, E, X, Result = G> + Clone,
        T: Transform<O, T, D, S, G, A, M, E, X, Result = A> + Clone,
        D: Transform<O, T, D, S, G, A, M, E, X, Result = M> + Clone,
        S: Transform<O, T, D, S, G, A, M, E, X, Result = E> + Clone,
        G: Clone,
        A: Default + Clone,
        M,
        E: Default + Clone,
        X,
    > Transformer<B, C, O, T, D, S, G, A, M, E, X>
{
    pub fn translate<F: Transform<O, T, D, S, G, A, M, E, X>>(
        &mut self,
        f: &F,
    ) -> anyhow::Result<F::Result> {
        return f.rewrite(
            self,
            |t, f| t.transform_func(f),
            |t, d| t.transform_datum(d),
        );
    }
    pub fn transform_datum(&mut self, d: Id<D>) -> anyhow::Result<Id<M>> {
        loop {
            if let Some(e) = self.dcache.get(&d) {
                return Ok(*e);
            }
            let dv = self.input.data[d].clone();
            let ev = self.translate(&dv)?;
            self.dcache.insert(d, self.output.data.alloc(ev));
        }
    }
    pub fn transform_func(
        &mut self,
        d: Id<crate::Func<O, T, D, S>>,
    ) -> anyhow::Result<Id<crate::Func<G, A, M, E>>> {
        loop {
            if let Some(e) = self.fcache.get(&d) {
                return Ok(*e);
            }
            let dv = self.input.funcs[d].clone();
            let mut n: Func<G, A, M, E> = Default::default();
            let a = self.output.funcs.alloc(n.clone());
            self.fcache.insert(d, a);
            for (k, v) in dv.values.iter() {
                n.values.insert(
                    k.clone(),
                    match v {
                        Value::Operator(o, a, b) => {
                            let mut c = vec![];
                            for b in b.iter() {
                                c.push(self.transform_datum(*b)?);
                            }
                            let o = self.translate(o)?;
                            Value::Operator(o, a.clone(), c)
                        }
                        Value::Param(p) => Value::Param(p.clone()),
                        Value::Alias(l) => Value::Alias(l.clone()),
                    },
                );
            }
            n.terminator = self.translate(&dv.terminator)?;
            n.sig = self.translate(&dv.sig)?;
            self.output.funcs[a] = n;
        }
    }
}