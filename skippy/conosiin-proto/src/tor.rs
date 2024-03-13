use std::{iter::empty, pin::Pin, sync::Arc};

use crate::{Proto, ARW};
use arti_client::{DataStream, StreamPrefs, TorClient};
use futures::AsyncRead;
use futures::{future::Either, AsyncWrite, FutureExt, Stream, StreamExt};
use tor_cell::relaycell::msg::Connected;
use tor_hscrypto::pk::HsId;
use tor_hsservice::{config::OnionServiceConfigBuilder, OnionServiceConfig, RunningOnionService};
use tor_rtcompat::Runtime;

pub struct V1Tor<R: Runtime> {
    tor: TorClient<R>,
    service: Pin<Box<dyn Stream<Item = DataStream> + Send>>,
    service_obj: Arc<RunningOnionService>,
}
impl<R: Runtime> V1Tor<R> {
    pub async fn new(r: R) -> anyhow::Result<Self> {
        let mut t = TorClient::with_runtime(r.clone())
            .create_bootstrapped()
            .await?;
        let (sh, st) = t.launch_onion_service(OnionServiceConfigBuilder::default().build()?)?;
        return Ok(V1Tor {
            tor: t,
            service_obj: sh,
            service: Box::pin(tor_hsservice::handle_rend_requests(st).flat_map_unordered(
                None,
                |a| {
                    Box::pin(a.accept(Connected::new_empty()))
                        .map(|a| match a {
                            Ok(a) => Either::Left(futures::stream::iter(vec![a])),
                            Err(_) => Either::Right(futures::stream::empty()),
                        })
                        .flatten_stream()
                },
            )),
        });
    }
}

#[async_trait::async_trait]
impl<R: Runtime> Proto for V1Tor<R> {
    type Id = HsId;
    fn id(&self) -> Self::Id {
        return self.service_obj.onion_name().unwrap();
    }
    async fn get<'i>(&mut self, id: &'i Self::Id) -> anyhow::Result<Pin<Box<dyn ARW + Send + 'i>>> {
        let s = self
            .tor
            .connect_with_prefs(
                id.to_string(),
                StreamPrefs::new()
                    .connect_to_onion_services(arti_client::config::BoolOrAuto::Explicit(true)),
            )
            .await?;
        return Ok(Box::pin(s));
    }
    async fn pull(&mut self) -> anyhow::Result<Vec<Pin<Box<dyn ARW + Send + 'static>>>> {
        let s = self.service.next().await;
        return Ok(s
            .into_iter()
            .map(|a| {
                let x: Pin<Box<dyn ARW + Send + 'static>> = Box::pin(a);
                x
            })
            .collect::<Vec<Pin<Box<dyn ARW + Send + 'static>>>>());
    }
}

