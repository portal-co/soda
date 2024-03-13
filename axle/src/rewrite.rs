use std::marker::PhantomData;
use std::sync::Arc;

use crate::builder::*;
use crate::*;

pub trait Rewrite<O, T, D, S, G, A, M, E, X> {
    type Meta;
    type Boot;
    fn boot<'a>(&'a self, params: &IndexSet<ParamID>) -> impl Builder<G, A, M, E, Result = Self::Boot> + 'a;
    fn operation<'a, 'b, 'c, 'd, 'e, 'f, C: Ctx<O, T, D, S, G, A, M, E, X>>(
        &'a self,
        boot: &'b Self::Boot,
        c: &'f mut C,
        func: impl FnMut(
            &'f mut C,
            &'a Self,
            Id<crate::Func<O, T, D, S>>,
        ) -> anyhow::Result<Id<crate::Func<G, A, M, E>>>,
        datum: impl FnMut(&'f mut C, &'a Self, Id<D>) -> anyhow::Result<Id<M>>,
        o: &O,
        data: &'c [Id<D>],
        args: &'d [&'e Self::Meta],
    ) -> (
        impl Builder<G, A, M, E, Result = Self::Meta> + 'a + 'b + 'c + 'd + 'e,
        &'f mut C,
    );
    fn datum<'a, 'f, C: Ctx<O, T, D, S, G, A, M, E, X>>(
        &'a self,
        c: &'f mut C,
        func: impl FnMut(
            &'f mut C,
            &'a Self,
            Id<crate::Func<O, T, D, S>>,
        ) -> anyhow::Result<Id<crate::Func<G, A, M, E>>>,
        datum: impl FnMut(&'f mut C, &'a Self, Id<D>) -> anyhow::Result<Id<M>>,
        d: &D,
    ) -> anyhow::Result<M>;
    fn param<'a, 'b, 'f, C: Ctx<O, T, D, S, G, A, M, E, X>>(
        &'a self,
        boot: &'b Self::Boot,
        c: &'f mut C,
        func: impl FnMut(
            &'f mut C,
            &'a Self,
            Id<crate::Func<O, T, D, S>>,
        ) -> anyhow::Result<Id<crate::Func<G, A, M, E>>>,
        datum: impl FnMut(&'f mut C, &'a Self, Id<D>) -> anyhow::Result<Id<M>>,
        param: ParamID,
    ) -> (
        impl Builder<G, A, M, E, Result = Self::Meta> + 'a + 'b,
        &'f mut C,
    );
    fn terminator<'a, 'b, 'f, C: Ctx<O, T, D, S, G, A, M, E, X>, N: Deref<Target = Self::Meta> + Clone>(
        &'a self,
        boot: &'b Self::Boot,
        c: &'f mut C,
        func: impl FnMut(
            &'f mut C,
            &'a Self,
            Id<crate::Func<O, T, D, S>>,
        ) -> anyhow::Result<Id<crate::Func<G, A, M, E>>>,
        datum: impl FnMut(&'f mut C, &'a Self, Id<D>) -> anyhow::Result<Id<M>>,
        terminator: &T,
        map: &BTreeMap<ValueID, N>,
    ) -> (impl Builder<G, A, M, E, Result = A> + 'a + 'b, &'f mut C);
    fn sig(&self, a: &S) -> anyhow::Result<E>;
}

pub struct Transformer<B, C, O, T, D, S, G, A, M, E, X, R> {
    pub input: B,
    pub output: C,
    pub fcache: BTreeMap<Id<crate::Func<O, T, D, S>>, Id<crate::Func<G, A, M, E>>>,
    pub dcache: BTreeMap<Id<D>, Id<M>>,
    pub user: X,
    pub phantom: PhantomData<fn(&R)>,
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
        R,
    > Ctx<O, T, D, S, G, A, M, E, X> for Transformer<B, C, O, T, D, S, G, A, M, E, X, R>
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
        O: Clone,
        T: Clone,
        D: Clone,
        S: Clone,
        G: Clone,
        A: Default + Clone,
        M,
        E: Default + Clone,
        X,
        R: Rewrite<O, T, D, S, G, A, M, E, X>,
    > Transformer<B, C, O, T, D, S, G, A, M, E, X, R>
{
    // pub fn translate<F: Transform<O, T, D, S, G, A, M, E, X>>(
    //     &mut self,
    //     f: &F,
    // ) -> anyhow::Result<F::Result> {
    //     return f.rewrite(
    //         self,
    //         |t, f| t.transform_func(f),
    //         |t, d| t.transform_datum(d),
    //     );
    // }
    pub fn transform_datum(&mut self, d: Id<D>, process: &R) -> anyhow::Result<Id<M>> {
        loop {
            if let Some(e) = self.dcache.get(&d) {
                return Ok(*e);
            }
            let dv = self.input.data[d].clone();
            let ev = process.datum(
                self,
                |c, x, f| c.transform_func(f, x),
                |c, x, d| c.transform_datum(d, x),
                &dv,
            )?;
            self.dcache.insert(d, self.output.data.alloc(ev));
        }
    }
    pub fn transform_func(
        &mut self,
        d: Id<crate::Func<O, T, D, S>>,
        process: &R,
    ) -> anyhow::Result<Id<crate::Func<G, A, M, E>>> {
        loop {
            if let Some(e) = self.fcache.get(&d) {
                return Ok(*e);
            }
            let dv = self.input.funcs[d].clone();
            let p = dv.params();
            let mut n: Func<G, A, M, E> = Default::default();
            let a = self.output.funcs.alloc(n.clone());
            self.fcache.insert(d, a);
            let (boot, mut a) = process.boot(&p).build(&mut self.output, a)?;
            let mut m: BTreeMap<ValueID, Arc<<R as Rewrite<O, T, D, S, G, A, M, E, X>>::Meta>> =
                BTreeMap::new();
            for (k, v) in dv.values.iter() {
                m.insert(
                    k.clone(),
                    match v {
                        Value::Operator(o, az, b) => {
                            let mut r = vec![];
                            for a in az {
                                r.push(&**m.get(a).context("in getting a value")?);
                            }
                            let o = process.operation(
                                &boot,
                                self,
                                |c, x, f| c.transform_func(f, x),
                                |c, x, d| c.transform_datum(d, x),
                                o,
                                &b,
                                &r,
                            );
                            let (v, b) = o.0.build(&mut o.1.output, a)?;
                            a = b;
                            Arc::new(v)
                        }
                        Value::Param(p) => {
                            let o = process.param(
                                &boot,
                                self,
                                |c, x, f| c.transform_func(f, x),
                                |c, x, d| c.transform_datum(d, x),
                                p.clone(),
                            );
                            let (v, b) = o.0.build(&mut o.1.output, a)?;
                            a = b;
                            Arc::new(v)
                        }
                        Value::Alias(l) => m.get(l).context("in getting a value")?.clone(),
                    },
                );
            }
            let o = process.terminator(
                &boot,
                self,
                |c, x, f| c.transform_func(f, x),
                |c, x, d| c.transform_datum(d, x),
                &dv.terminator,
                &m,
            );
            let (v, b) = o.0.build(&mut o.1.output, a)?;
            a = b;
            n.terminator = v;
            n.sig = process.sig(&dv.sig)?;
            o.1.output.funcs[a] = n;
        }
    }
}
