use std::{env, path::Path};

use anyhow::Context;
use ed25519_dalek::{SecretKey, SigningKey};
enum Type{
    Encryption,Signing
}
fn main() -> anyhow::Result<()>{
    let mut args = std::env::args();
    args.next();
    let ty = match args.next().context("in getting the type")?.as_str(){
        "encryption" => Type::Encryption,
        "signing" => Type::Signing,
        _ => anyhow::bail!("invalid type"),
    };
    let private = args.next().context("in getting the key path")?;
    if Path::new(private.as_str()).exists(){
        eprintln!("private kry already exists, skipping");
        return Ok(());
    }
    let public = format!("{private}.pub");
    match ty{
        Type::Encryption => {
            let mut r = rand::random();
            std::fs::write(private, r)?;
            r = simple_encryption::x25519_base(r);
            std::fs::write(public, r)?;
        },
        Type::Signing => {
            let s = SigningKey::from_bytes(&rand::random());
            std::fs::write(private, s.as_bytes())?;
            std::fs::write(public, s.verifying_key().as_bytes())?;
        },
    }
    return Ok(());
}
