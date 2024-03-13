use crate::{add_start, i2x, new_sig, passes::mem_fusing::get_exports, rewrite_mem, x2i};
use paste::paste;
use std::{
    backtrace, borrow::Cow, collections::{BTreeMap, BTreeSet, HashMap}, hash::Hash, iter::empty, ops::{Deref, DerefMut}
};

use waffle::{entity::EntityRef, ExportKind, Func, FuncDecl, FunctionBody, Global, ImportKind, Memory, Module, Operator, Signature, SignatureData, Table, TableData, ValueDef};
#[derive(Eq, PartialEq, Clone)]
pub struct IKW(pub ImportKind);
impl Hash for IKW {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match &self.0 {
            ImportKind::Table(a) => {
                state.write_usize(0);
                a.hash(state);
            }
            ImportKind::Func(a) => {
                state.write_usize(1);
                a.hash(state);
            }
            ImportKind::Global(a) => {
                state.write_usize(2);
                a.hash(state);
            }
            ImportKind::Memory(a) => {
                state.write_usize(3);
                a.hash(state);
            }
        }
    }
}

pub struct State<I> {
    cache: HashMap<IKW, ImportKind>,
    fun_cache: BTreeMap<Func, Func>,
    table_cache: BTreeMap<Table, Table>,
    pub importmap: I,
    pub tables: BTreeSet<Table>,
}
impl<I> State<I> {
    pub fn new(importmap: I,tables: BTreeSet<Table>) -> Self {
        return Self {
            importmap,
            cache: Default::default(),
            fun_cache: Default::default(),
            table_cache: Default::default(),
            tables
        };
    }
}
pub struct Copier<A, B,S> {
    pub src: A,
    pub dest: B,
    pub state: S,
}
macro_rules! translator {
    ($a:tt) => {
        paste! {
            pub fn [<translate_ $a>](&mut self, m: $a) -> anyhow::Result<$a>{
                let ImportKind::$a(n) = self.translate_import(ImportKind::$a(m))? else{
                    anyhow::bail!("not the right item")
                };
                return Ok(n);
            }
        }
    };
}
pub trait Imports{
    fn get_import(&mut self,a: &mut Module<'static>,m: String,n: String) -> anyhow::Result<Option<ImportBehavior>>;
}
#[repr(transparent)]
#[derive(Clone,Debug,Copy)]
pub struct ImportFn<F>(pub F);

impl<F: FnMut(&mut Module<'static>,String,String) -> anyhow::Result<Option<ImportBehavior>>> Imports for ImportFn<F>{
    fn get_import(&mut self,a: &mut Module<'static>,m: String,n: String) -> anyhow::Result<Option<ImportBehavior>> {
        return (self.0)(a,m,n);
    }
}

pub enum ImportBehavior{
    Bind(ImportKind),
    Passthrough(String,String),
}
impl<A: Deref<Target = waffle::Module<'static>>, B: Deref<Target = waffle::Module<'static>> + DerefMut,S: Deref<Target = State<I>> + DerefMut,I: Deref<Target = J> + DerefMut,J: Imports> Copier<A, B,S> {
    pub fn collect(&mut self) -> anyhow::Result<BTreeMap<String,ExportKind>>{
        let e = get_exports(&self.src);
        // let mut m = HashMap::new();
        let mut m = BTreeMap::new();
        for (s,e) in e{
            let e = x2i(e);
            let r = self.translate_import(e.clone())?;
            m.insert(s, i2x(r));
        }
        return Ok(m);
    }
    pub fn new(
        src: A,
        dest: B,
        // importmap: BTreeMap<(String, String), ImportKind>,
        cache: S,
    ) -> Self {
        return Self {
            src,
            dest,
            // importmap,
            state: cache,
        };
    }
    pub fn resolve_import(&mut self, a: &ImportKind) -> anyhow::Result<Option<ImportBehavior>> {
        for i in self.src.imports.iter() {
            if i.kind == *a {
                return self
                    .state
                    .importmap.get_import(&mut self.dest,i.module.clone(),i.name.clone())
            }
        }
        return Ok(None);
    }
    pub fn internal_translate_mem(&mut self, a: waffle::Memory) -> anyhow::Result<waffle::Memory> {
        let d = self.src.memories[a].clone();
        return Ok(self.dest.memories.push(d));
    }
    pub fn internal_translate_global(
        &mut self,
        a: waffle::Global,
    ) -> anyhow::Result<waffle::Global> {
        let d = self.src.globals[a].clone();
        return Ok(self.dest.globals.push(d));
    }
    pub fn translate_import(&mut self, a: ImportKind) -> anyhow::Result<ImportKind> {
        let i = match self.resolve_import(&a)?{
            None => None,
            Some(ImportBehavior::Bind(b)) => return Ok(b),
            Some(ImportBehavior::Passthrough(a, b)) => Some((a,b))
        };
        if let Some(b) = self.state.cache.get(&IKW(a.clone())) {
            return Ok(b.clone());
        }
        let c = match a {
            ImportKind::Table(t) => ImportKind::Table(self.internal_translate_table(t)?),
            ImportKind::Func(f) => ImportKind::Func(self.internal_translate_func(f)?),
            ImportKind::Global(g) => ImportKind::Global(self.internal_translate_global(g)?),
            ImportKind::Memory(m) => ImportKind::Memory(self.internal_translate_mem(m)?),
        };
        self.state.cache.insert(IKW(a.clone()), c.clone());
        if let Some(j) = i{
            self.dest.imports.push(waffle::Import { module: j.0, name: j.1, kind: c.clone() })
        }
        return Ok(c);
    }
    pub fn internal_translate_table(&mut self, tk: Table) -> anyhow::Result<Table> {
        // if let Some(c) = self.state.table_cache.get(&tk) {
        //     return Ok(*c);
        // }
        let mut t = self.src.tables[tk].clone();
        // let nt = self.dest.tables.push(t.clone());
        // self.state.table_cache.insert(tk, nt);
        if let Some(u) = t.func_elements.as_mut() {
            for w in u.iter_mut() {
                *w = self.translate_Func(*w)?;
            }
        }
        // self.dest.tables[nt] = t;
        return Ok(self.dest.tables.push(t));
    }
    pub fn translate_sig(&mut self, s: Signature) -> anyhow::Result<Signature> {
        let d = self.src.signatures[s].clone();
        return Ok(new_sig(&mut *self.dest, d));
    }
    pub fn internal_translate_func(&mut self, f: Func) -> anyhow::Result<Func> {
        if f == Func::invalid(){
            return Ok(f);
        }
        if let Some(x) = self.state.fun_cache.get(&f) {
            return Ok(*x);
        }
        let a = self.dest.funcs.push(waffle::FuncDecl::None);
        self.state.fun_cache.insert(f, a);
        for t in &self.state.tables{
            self.dest.tables[*t].func_elements.get_or_insert(vec![]).push(a);
        }
        if Some(f) == self.src.start_func{
           add_start(&mut self.dest, a); 
        }
        let mut f = self.src.funcs[f].clone();
        if let Some(b) = f.body_mut() {
            for v in b.values.iter() {
                let mut k = b.values[v].clone();
                if let ValueDef::Operator(a, vs, c) = &mut k {
                    let mut w = b.arg_pool[*vs].to_vec();
                    let mut e = None;
                    rewrite_mem(a, &mut w, |m, _| {
                        *m = match self.translate_Memory(*m) {
                            Ok(n) => n,
                            Err(o) => {
                                e = Some(o);
                                *m
                            }
                        };
                    });
                    if let Some(e) = e {
                        return Err(e);
                    }
                    match a {
                        waffle::Operator::Call { function_index } => {
                            *function_index = self.translate_Func(*function_index)?;
                        }
                        waffle::Operator::CallIndirect {
                            sig_index,
                            table_index,
                        } => {
                            *sig_index = self.translate_sig(*sig_index)?;
                            *table_index = self.translate_Table(*table_index)?;
                        }
                        waffle::Operator::GlobalGet { global_index } => {
                            *global_index = self.translate_Global(*global_index)?;
                        }
                        waffle::Operator::GlobalSet { global_index } => {
                            *global_index = self.translate_Global(*global_index)?;
                        }
                        waffle::Operator::TableGet { table_index } => {
                            *table_index = self.translate_Table(*table_index)?;
                        }
                        waffle::Operator::TableSet { table_index } => {
                            *table_index = self.translate_Table(*table_index)?;
                        }
                        waffle::Operator::TableGrow { table_index } => {
                            *table_index = self.translate_Table(*table_index)?;
                        }
                        waffle::Operator::TableSize { table_index } => {
                            *table_index = self.translate_Table(*table_index)?;
                        }
                        _ => {}
                    }
                    *vs = b.arg_pool.from_iter(w.into_iter());
                }
                b.values[v] = k;
            }
            for k in b.blocks.iter() {
                let mut kv = b.blocks[k].clone();
                match &mut kv.terminator {
                    waffle::Terminator::ReturnCall { func, args } => {
                        *func = self.translate_Func(*func)?;
                    }
                    waffle::Terminator::ReturnCallIndirect { sig, table, args } => {
                        *sig = self.translate_sig(*sig)?;
                        *table = self.translate_Table(*table)?;
                    }
                    _ => {}
                }
                b.blocks[k] = kv;
            }
            // (self.state.instrument)(&mut self.dest,b)?;
        }
        match &mut f {
            waffle::FuncDecl::Import(a, _) => {
                *a = self.translate_sig(*a)?;
            }
            waffle::FuncDecl::Lazy(_, _, _) => todo!(),
            waffle::FuncDecl::Body(a, _, _) => {
                *a = self.translate_sig(*a)?;
            }
            waffle::FuncDecl::Compiled(_, _, _) => todo!(),
            waffle::FuncDecl::None => {}
        }
        self.dest.funcs[a] = f;
        return Ok(a);
    }
    translator!(Memory);
    translator!(Table);
    translator!(Global);
    translator!(Func);
}

#[cfg(test)]
mod tests {
    use super::*;
}
