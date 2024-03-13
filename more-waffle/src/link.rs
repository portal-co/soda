use std::{
    collections::{BTreeMap, BTreeSet},
    ops::{Deref, DerefMut},
};

use waffle::Module;

use crate::{copying::module::{Copier, Imports, State}, passes::mem_fusing::get_exports, x2i};

pub struct Linker<A> {
    pub modules: A,
}
impl<
        K: Eq + Ord,
        A: Deref<Target = BTreeMap<K, Module<'static>>>,
    > Imports for Linker<A>
{
    fn get_import(
        &mut self,
        a: &mut Module<'static>,
        m: String,
        n: String,
    ) -> anyhow::Result<Option<crate::copying::module::ImportBehavior>> {
        if m == "env" {
            for mo in self.modules.values() {
                let e = get_exports(mo);
                if let Some(e) = e.get(&n){
                    let e = e.clone();
                    let e = x2i(e);
                    let mo = mo.clone();
                    let e = Copier::new(&mo, &mut *a, Box::new(State::new(&mut *self, BTreeSet::new()))).translate_import(e)?;
                    return Ok(Some(crate::copying::module::ImportBehavior::Bind(e)));
                }
            }
        }
        return Ok(Some(crate::copying::module::ImportBehavior::Passthrough(
            m, n,
        )));
    }
}
