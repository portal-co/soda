use syn_serde_struct::Syn;
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Syscall{
    pub id: [u8; 32],
    pub poke_data: Syn<proc_macro2::TokenStream>,
    pub poke_trait: Syn<proc_macro2::TokenStream>,
    pub handler: Syn<proc_macro2::TokenStream>
}

#[cfg(test)]
mod tests {
    use super::*;

}
