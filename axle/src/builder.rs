use crate::*;

pub trait Builder<O, T, D, S> {
    type Result;
    fn build(
        &self,
        module: &mut Module<O, T, D, S>,
        func: Id<Func<O, T, D, S>>,
    ) -> anyhow::Result<(Self::Result, Id<Func<O, T, D, S>>)>;
    fn flat_map<R: Builder<O, T, D, S>, F: Fn(Self::Result) -> R>(self, f: F) -> FlatMap<Self, F>
    where
        Self: Sized,
    {
        return FlatMap {
            wrapped: self,
            func: f,
        };
    }
}
pub struct BuildFn<F> {
    pub func: F,
}
pub fn build_fn<F>(f: F) -> BuildFn<F> {
    return BuildFn { func: f };
}
impl<
        O,
        T,
        D,
        S,
        X,
        F: for<'a> Fn(
            &'a mut Module<O, T, D, S>,
            Id<Func<O, T, D, S>>,
        ) -> anyhow::Result<(X, Id<Func<O, T, D, S>>)>,
    > Builder<O, T, D, S> for BuildFn<F>
{
    type Result = X;

    fn build(
        &self,
        module: &mut Module<O, T, D, S>,
        func: Id<Func<O, T, D, S>>,
    ) -> anyhow::Result<(Self::Result, Id<Func<O, T, D, S>>)> {
        return (self.func)(module, func);
    }
}
impl<O, T, D, S, X: Builder<O, T, D, S> + ?Sized> Builder<O, T, D, S> for Box<X> {
    type Result = X::Result;

    fn build(
        &self,
        module: &mut Module<O, T, D, S>,
        func: Id<Func<O, T, D, S>>,
    ) -> anyhow::Result<(Self::Result, Id<Func<O, T, D, S>>)> {
        return (&**self).build(module, func);
    }
}
pub struct FlatMap<B, F> {
    pub wrapped: B,
    pub func: F,
}
impl<O, T, D, S, B: Builder<O, T, D, S>, F: Fn(B::Result) -> X, X: Builder<O, T, D, S>>
    Builder<O, T, D, S> for FlatMap<B, F>
{
    type Result = X::Result;

    fn build(
        &self,
        module: &mut Module<O, T, D, S>,
        func: Id<Func<O, T, D, S>>,
    ) -> anyhow::Result<(Self::Result, Id<Func<O, T, D, S>>)> {
        let (a, func) = self.wrapped.build(module, func)?;
        return (self.func)(a).build(module, func);
    }
}
