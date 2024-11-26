use anyhow::Result;
use quartz_common::enclave::{
    attestor::Attestor,
    server::{WebSocketHandler, WsListenerConfig},
};
use serde::Deserialize;
use tendermint_rpc::event::Event;
use crate::fairblock_server::FairblockService;

#[async_trait::async_trait]
impl<A: Attestor + Clone> WebSocketHandler for FairblockService<A> {
    async fn handle(&self, _event: Event, _config: WsListenerConfig) -> Result<()> {
        Ok(())
    }
}

#[tonic::async_trait]
pub trait WsListener: Send + Sync + 'static {}

#[async_trait::async_trait]
impl<A> WsListener for FairblockService<A>
where
    A: Attestor,
    A::RawAttestation: for<'de> Deserialize<'de> + Send,
{
}
