use std::mem::take;

use anyhow::Context;
use more_waffle::passes::mem_fusing::get_exports;
use waffle::{ExportKind, FuncDecl, FunctionBody, ImportKind};

pub fn icify(m: &mut waffle::Module, prefix: &str) -> anyhow::Result<()> {
    let ex = get_exports(m);
    for i in take(&mut m.imports) {
        if i.module == "wasi_snapshot_preview1" || i.module == "wasi_unstable" {
            let ImportKind::Func(f) = i.kind else {
                continue;
            };
            let ic = ex
                .get(&format!("{prefix}{}", i.name))
                .context("in getting export")?
                .clone();
            let ExportKind::Func(ic) = ic else {
                anyhow::bail!("wrong type");
            };
            let sig = m.funcs[f].sig();
            let mut new = FunctionBody::new(&m, sig);
            let params = new.blocks[new.entry].params.iter().map(|a| a.1).collect();
            new.set_terminator(
                new.entry,
                waffle::Terminator::ReturnCall {
                    func: ic,
                    args: params,
                },
            );
            m.funcs[f] = FuncDecl::Body(sig, m.funcs[f].name().to_owned(), new);
        } else {
            m.imports.push(i);
        }
    }
    if let Some(ExportKind::Func(s)) = ex.get("_initialize") {
        more_waffle::add_start(m, *s)
    }
    if let Some(ExportKind::Func(s)) = ex.get("_start") {
        more_waffle::add_start(m, *s)
    }
    return Ok(());
}
#[cfg(test)]
mod tests {
    use super::*;
}
