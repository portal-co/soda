use std::collections::BTreeMap;

use paxel_common::Syscall;
use syn::Ident;
use syn_serde_struct::Syn;

pub fn syscalls(m: &waffle::Module) -> anyhow::Result<Vec<paxel_common::Syscall>> {
    let mut w = vec![paxel_common::Syscall {
        id: [0u8; 32],
        poke_data: Syn(quote::quote! {
            streams: BTreeMap<u32,X::Stream>,
        }),
        poke_trait: Syn(quote::quote! {
            type Stream: ::futures::io::AsyncRead + ::futures::io::AsyncWrite + Send + Sync + 'static;
        }),
        handler: Syn(quote::quote! {
            match input.get(0){
                Some(b's') => {
                    let s = self.streams.get_mut(u32::from_le_bytes(&input[1..6])).context("in getting stream")?;
                    match input.get(6){
                        Some(b'w') => {
                            return Ok(vec![0; s.write(&input[7..]).await?]);
                        },
                        Some(b'r') => {
                            let mut w = input[7..].to_vec();
                            let a = s.read(&mut w).await?;
                            let w = w[..a],to_vec();
                            return Ok(w);
                        },
                        _ => anyhow::bail!("not supported")
                    }
                },
                _ => anyhow::bail!("not supported")
            }
        }),
    }];
    for (k, v) in &m.custom_sections {
        if k.starts_with(".paxel_syscall") {
            let p = postcard::from_bytes(&v)?;
            w.push(p);
        }
    }
    return Ok(w);
}
#[derive(Clone)]
pub struct Opts {
    pub inherit: Ident,
    pub bake_syscalls: Vec<Syscall>,
}
pub fn paxel_wars(
    m: waffle::Module,
    target: Ident,
    opts: &Opts,
) -> anyhow::Result<proc_macro2::TokenStream> {
    let inherit = opts.inherit.clone();
    let mut sy = syscalls(&m)?;
    sy.extend(opts.bake_syscalls.iter().map(|a| a.clone()));
    let cor = Ident::new(&format!("{target}Core"), target.span());
    let dat = Ident::new(&format!("{target}Data"), target.span());
    let coredat = Ident::new(&format!("{target}CoreData"), target.span());
    let w = wars_core::lower_module(
        m,
        cor.clone(),
        &wars_core::Opts {
            r#async: true,
            result: true,
            serde: false,
            imports: BTreeMap::new(),
            inherit: target.clone(),
            r#impl: false,
        },
    )?;
    let sh = sy.iter().map(|a| {
        let i: &[u8] = &a.id;
        let j = a.handler.clone().0;
        quote::quote! {
            [#(#i),*] => #j,
        }
    });
    let sd = sy.iter().map(|d| d.poke_data.0.clone());
    let st = sy.iter().map(|h| h.poke_trait.0.clone());
    return Ok(quote::quote! {
        #w
        pub fn alloc<T>(m: &mut BTreeMap<u32,T>, x: T) -> u32{
            let mut u = 0;
            while m.contains(&u){
                u += 1;
            };
            m.insert(u,x);
            return u;
        }

        #[derive(Default)]
        pub struct #dat<X: #target>{
            core: #coredat,
            #(#sd)*
        }
        #[async_trait::async_trait]
        pub trait #target: #inherit{
            #(#st)*
            fn y(&mut self) -> &mut #dat<Self>;
            async fn syscall(&mut self,a: Vec<u8>) -> anyhow::Result<Vec<u8>>{
                let ssn: [u8; 32]= a[0..32].try_into()?;
                let input = a[32..].to_vec();
                match ssn{
                    #(#sh)*
                    _ => anyhow::bail!("not supported")
                }
            }
        }
        #[async_trait::async_trait]
        impl<T: #target + Send + Sync> #cor for T{
            fn z(&mut self) -> &mut #coredat{
                return &mut self.y().core;
            }
            async fn i_syscall(&mut self, a: u32) -> anyhow::Result<(u32,)>{
                let (l,) = self.len_of(a).await?;
                let s = self.memory()[(a as usize)..][..(l as usize)].to_vec();
                let s = self.syscall(s).await?;
                self.free(a).await?;
                let (m,) = self.malloc(s.len() as u32).await?;
                self.memory()[(m as usize)..][..s.len()].copy_from_slice(&s);
                return Ok((m,));
            }
        }

    });
}

#[cfg(test)]
mod tests {
    use super::*;
}
