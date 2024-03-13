use paxel_common::Syscall;
use sha3::Digest;

pub fn write_raw_syscall(a: Syscall) -> anyhow::Result<proc_macro2::TokenStream> {
    let b = postcard::to_allocvec(&a)?;
    let l = b.len();
    let h = sha3::Sha3_256::digest(&b);
    let hs: [u8; 32] = h.as_slice().try_into()?;
    let hx = format!("paxel_syscall_{:?}", hs);
    return Ok(quote::quote! {
        const _: () = {
            #[link_section = #hx]
            static A: [u8; #l] = [#(#b),*];
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
}
