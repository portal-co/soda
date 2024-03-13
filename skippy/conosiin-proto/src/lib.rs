use std::pin::Pin;

use futures::{AsyncRead, AsyncWrite};
#[cfg(feature = "tor")]
pub mod tor;
pub trait ARW: AsyncRead + AsyncWrite{}
impl<T: AsyncRead + AsyncWrite> ARW for T{}

#[async_trait::async_trait]
pub trait Proto{
    type Id: Clone;
    fn id(&self) -> Self::Id;
    async fn get<'i>(&mut self, id: &'i Self::Id) -> anyhow::Result<Pin<Box<dyn ARW + Send + 'i>>>;
    async fn pull(&mut self) -> anyhow::Result<Vec<Pin<Box<dyn ARW + Send + 'static>>>>;
}



#[cfg(test)]
mod tests {
    use super::*;

}
