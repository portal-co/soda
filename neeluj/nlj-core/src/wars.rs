use std::collections::BTreeMap;
use anyhow::Context;
use futures::AsyncReadExt;
use futures::AsyncWriteExt;
pub fn create<T>(m: &mut BTreeMap<u32,T>, x: T) -> u32{
    let mut u = 0;
    while m.contains_key(&u){
        u += 1;
    };
    m.insert(u,x);
    return u;
}
wars::wars!("target/wasm32-unknown-unknown/release/nlj_x.wasm" => XCore: X "ar");
#[derive(Default)]
pub struct XData<T: X + ?Sized>{
    core: XCoreData,
    streams: BTreeMap<u32,T::Stream>,
}
#[async_trait::async_trait]
pub trait X: Send + Sync{
    type Stream: Send + Sync + 'static + futures::AsyncRead + futures::AsyncWrite + Unpin;
    fn data(&mut self) -> &mut XData<Self>;
    async fn alloc(&mut self, a: Vec<u8>) -> anyhow::Result<u32>{
        let (m,) = <Self as XCore>::malloc(self,a.len() as u32).await?;
        <Self as XCore>::memory(self)[(m as usize)..][..a.len()].copy_from_slice(&a);
        return Ok(m)
    }
    async fn take(&mut self, a: u32) -> anyhow::Result<Vec<u8>>{
        let (l,) = <Self as XCore>::len_of(self,a).await?;
        let n = <Self as XCore>::memory(self)[(a as usize)..][..(l as usize)].to_vec();
        <Self as XCore>::free(self,a).await?;
        return Ok(n);
    }
    async fn syscall(&mut self, mut a: Vec<u8>) -> anyhow::Result<Vec<u8>>{
        match a.get(0){
            Some(b's') => {
                let (b,a) = a.split_at_mut(4);
                let mut s = self.data().streams.get_mut(&u32::from_le_bytes(b[1..].try_into()?)).context("in getting a stream")?;
                match a.get(0){
                    Some(b'r') => {
                        let n = s.read(&mut a[1..]).await?;
                        return Ok(a[1..][..n].to_vec())
                    },
                    Some(b'w') => {
                        let n = s.write(&a[1..]).await?;
                        return Ok(vec![0u8; n]);
                    },
                    _ => anyhow::bail!("not supported")
                }
            },
            _ => anyhow::bail!("not supported")
        }
    }
}
#[async_trait::async_trait]
impl<T: X + ?Sized> XCore for T{
    fn z(&mut self) -> &mut XCoreData{
        return &mut self.data().core;
    }
    async fn i_syscall(&mut self, a: u32) -> anyhow::Result<(u32,)>{
        let t = self.take(a).await?;
        let s = self.syscall(t).await?;
        return Ok((self.alloc(s).await?,))
    }
}



