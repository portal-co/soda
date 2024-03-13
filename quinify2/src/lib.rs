// use std::iter::empty;

// use waffle::{
//     BlockTarget, Func, FunctionBody, MemoryData, MemorySegment, Module, Operator, SignatureData,
//     Type, MemoryArg,
// };
use std::{collections::BTreeMap, io::Write, iter::empty};

use waffle::{
    wasmparser::types::KebabStr, Block, ExportKind, Func, FuncDecl, FunctionBody, Import,
    ImportKind, Memory, MemoryArg, MemoryData, Module, Operator, Signature, SignatureData, Table,
    Type, Value, ValueDef,
};
pub fn new_sig(m: &mut Module, s: SignatureData) -> Signature {
    for (a, b) in m.signatures.entries() {
        if *b == s {
            return a;
        }
    }
    return m.signatures.push(s);
}
pub fn add_start(m: &mut Module, tf: Func) {
    let s = SignatureData {
        params: vec![],
        returns: vec![],
    };
    let s = new_sig(m, s);
    let mut f = FunctionBody::new(&m, s);
    let vz = f.arg_pool.from_iter(empty());
    let t = m.funcs[tf].sig();
    let t = m.signatures[t].clone().returns;
    let tz = f.type_pool.from_iter(t.into_iter());
    let v = f.add_value(ValueDef::Operator(
        Operator::Call { function_index: tf },
        vz,
        tz,
    ));
    f.append_to_block(f.entry, v);
    f.set_terminator(
        f.entry,
        match m.start_func {
            Some(a) => waffle::Terminator::ReturnCall {
                func: a,
                args: vec![],
            },
            None => waffle::Terminator::Return { values: vec![] },
        },
    );
    let f = m.funcs.push(FuncDecl::Body(s, format!("start"), f));
    m.start_func = Some(f);
}
pub fn quin_iter(m: &mut Module, x: impl Iterator<Item = (u8)>, q: Func) {
    let null = new_sig(
        m,
        SignatureData {
            params: vec![],
            returns: vec![],
        },
    );
    let mut b = FunctionBody::new(m, null);
    let vz = b.arg_pool.from_iter(empty());
    let tz = b.type_pool.from_iter(empty());
    let ti = b.type_pool.from_iter(vec![Type::I32].into_iter());
    let ia = b.add_value(ValueDef::Operator(Operator::I32Const { value: 0 }, vz, ti));
    b.append_to_block(b.entry, ia);
    for c in x {
        // let ia = b.add_value(ValueDef::Operator(Operator::I32Const { value: a as u32 }, vz, ti));
        // b.append_to_block(b.entry, ia);
        let ic = b.add_value(ValueDef::Operator(
            Operator::I32Const { value: c as u32 },
            vz,
            ti,
        ));
        b.append_to_block(b.entry, ic);
        let vs = b.arg_pool.from_iter(vec![ia, ic].into_iter());
        let j = b.add_value(ValueDef::Operator(
            Operator::Call { function_index: q },
            vs,
            tz,
        ));
        b.append_to_block(b.entry, j);
    }
    b.set_terminator(b.entry, waffle::Terminator::Return { values: vec![] });
    let f = m.funcs.push(FuncDecl::Body(null, format!("z"), b));
    add_start(m, f);
}
pub fn metaquin_iter(m: &mut Module, x: &[(u8)], q: Func) {
    for w in x.chunks(4096) {
        quin_iter(m, w.iter().map(|a| *a), q);
    }
}
struct Quin<'a, 'b> {
    module: &'a mut Module<'b>,
    func: Func,
    buffer: Vec<u8>,
}
impl<'a, 'b> Write for Quin<'a, 'b> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend(buf.iter().map(|a|*a));
        if self.buffer.len() >= 4096 {
            quin_iter(self.module, self.buffer.drain(..), self.func);
        }
        return Ok(buf.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}
pub trait ModuleQuinifyExtras{
    fn quinify(&mut self, func: Func) -> impl Write;
}
// pub fn metafuse(m: &mut Module, mem: Memory, dat: MemoryData){
//     let null = new_sig(
//         m,
//         SignatureData {
//             params: vec![],
//             returns: vec![],
//         },
//     );
//     let mut v = vec![];
//     for s in dat.segments.iter(){
//         v.extend(s.data.iter().enumerate().map(|(a,b)|(a + s.offset,*b)));
//     }
//     metafuse_iter(m, &v, mem);
//     let mut b = FunctionBody::new(m, null);
//     let vz = b.arg_pool.from_iter(empty());
//     let tz = b.type_pool.from_iter(empty());
//     let ti = b.type_pool.from_iter(vec![Type::I32].into_iter());
//     let ia = b.add_value(ValueDef::Operator(Operator::I32Const { value: dat.initial_pages as u32 }, vz, ti));
//     b.append_to_block(b.entry, ia);
//     let vs = b.arg_pool.from_iter(vec![ia].into_iter());
//     let ib = b.add_value(ValueDef::Operator(Operator::MemoryGrow { mem: mem },vs, tz));
//     b.append_to_block(b.entry, ib);
//     b.set_terminator(b.entry, waffle::Terminator::Return { values: vec![] });
//     let f = m.funcs.push(FuncDecl::Body(null, format!("z"), b));
//     add_start(m, f);
// }
// pub fn metafuse_all(m: &mut Module){
//     let mut b = BTreeMap::new();
//     for mem in m.memories.entries_mut(){
//         b.insert(mem.0, std::mem::replace( mem.1,MemoryData { initial_pages: 0, maximum_pages: None, segments: vec![] }));
//     }
//     for (c,d) in b.into_iter(){
//         metafuse(m, c, d);
//     }
// }
#[cfg(test)]
mod tests {
    use super::*;
}

// pub fn quinify2(m: &mut Module, q: Func, data: Option<Vec<u8>>) -> anyhow::Result<()> {
//     let e = match data{
//         None => m.to_wasm_bytes()?,
//         Some(e) => e,
//     };
//     let l = e.len();
//     let p = (e.len() + 65535) / 65536;
//     let s = m.signatures.push(SignatureData {
//         params: vec![],
//         returns: vec![],
//     });
//     let mut new = FunctionBody::new(&m, s);
//     let vz = new.arg_pool.from_iter(empty());
//     let tz = new.type_pool.from_iter(empty());
//     let mem = m.memories.push(MemoryData {
//         initial_pages: p,
//         maximum_pages: Some(p),
//         segments: vec![MemorySegment { offset: 0, data: e }],
//     });
//     // Final block
//     let rb = new.add_block();
//     new.set_terminator(
//         rb,
//         match m.start_func {
//             None => waffle::Terminator::Return { values: vec![] },
//             Some(a) => waffle::Terminator::ReturnCall {
//                 func: a,
//                 args: vec![],
//             },
//         },
//     );
//     // Start of copy loop
//     let mb = new.add_block();
//     let idx = new.add_blockparam(mb, Type::I32);
//     let ts = new.type_pool.from_iter(vec![Type::I32].into_iter());
//     let a = new.add_value(waffle::ValueDef::Operator(
//         Operator::I32Const { value: l as u32 },
//         vz,
//         ts,
//     ));
//     new.append_to_block(mb, a);
//     let vs = new.arg_pool.from_iter(vec![a, idx].into_iter());
//     let b = new.add_value(waffle::ValueDef::Operator(Operator::I32Eq, vs, ts));
//     new.append_to_block(mb, b);
//     // Other part of copy loop
//     let nb = new.add_block();
//     new.set_terminator(
//         mb,
//         waffle::Terminator::CondBr {
//             cond: b,
//             if_true: BlockTarget {
//                 block: rb,
//                 args: vec![],
//             },
//             if_false: BlockTarget {
//                 block: nb,
//                 args: vec![],
//             },
//         },
//     );
//     let vs = new.arg_pool.from_iter(vec![a].into_iter());
//     let rdx = new.add_value(waffle::ValueDef::Operator(Operator::I32Load8U { memory: MemoryArg{align: 0, offset: 0, memory: mem} }, vs,ts));
//     new.append_to_block(nb, rdx);
//     let vs = new.arg_pool.from_iter(vec![rdx].into_iter());
//     let cl = new.add_value(waffle::ValueDef::Operator(Operator::Call { function_index: q }, vs, tz));
//     new.append_to_block(nb, cl);
//     let a = new.add_value(waffle::ValueDef::Operator(
//         Operator::I32Const { value:1},
//         vz,
//         ts,
//     ));
//     new.append_to_block(nb, a);
//     let vs = new.arg_pool.from_iter(vec![a, idx].into_iter());
//     let b = new.add_value(waffle::ValueDef::Operator(Operator::I32Add, vs, ts));
//     new.append_to_block(nb, b);
//     new.set_terminator(nb, waffle::Terminator::Br { target: BlockTarget{block: mb, args: vec![b]} });
//     //Fix up
//     let a = new.add_value(waffle::ValueDef::Operator(
//         Operator::I32Const { value:0},
//         vz,
//         ts,
//     ));
//     new.append_to_block(new.entry, a);
//     new.set_terminator(new.entry, waffle::Terminator::Br { target: BlockTarget{block: mb, args: vec![a]} });
//     //Seal the deal
//     let f = m.funcs.push(waffle::FuncDecl::Body(s, "$quinify".to_owned(), new));
//     m.start_func = Some(f);
//     return Ok(());
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
// }
