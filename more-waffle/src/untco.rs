use std::collections::BTreeMap;

use bimap::BiBTreeMap;
use waffle::{Block, Func, Module, SignatureData, Terminator};

use crate::new_sig;

pub fn untco(m: &mut Module) -> BiBTreeMap<(Func,Block),Func>{
    let _s: *mut Module = m;
    let mut nm = BTreeMap::new();
    for (fi, f) in m.funcs.entries().collect::<Vec<_>>() {
        if let Some(b) = f.body() {
            for k in b.blocks.iter() {
                if k == b.entry {
                    nm.insert((fi, k), fi);
                } else {
                    let mut n = b.clone();
                    n.entry = k;
                    let ns = new_sig(
                        unsafe { &mut *_s },
                        SignatureData {
                            params: n.blocks[k].params.iter().map(|a| a.0).collect(),
                            returns: n.rets.clone(),
                        },
                    );
                    let n = unsafe { &mut *_s }.funcs.push(waffle::FuncDecl::Body(
                        ns,
                        format!("{}.~{}", f.name(), k),
                        n,
                    ));
                    nm.insert((fi, k), n);
                }
            }
        }
    }
    for ((pf, pb), tf) in nm.iter() {
        if let Some(b) = m.funcs[*tf].body_mut() {
            for (k, kd) in b.blocks.entries_mut() {
                if k == *pb {
                    continue;
                }
                let f = *nm.get(&(*pf, k)).unwrap();
                kd.insts = vec![];
                kd.terminator = Terminator::ReturnCall {
                    func: f,
                    args: kd.params.iter().map(|a| a.1).collect(),
                }
            }
            b.recompute_edges();
        }
    };
    return nm.into_iter().collect();
}
