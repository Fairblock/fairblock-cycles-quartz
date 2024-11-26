use std::sync::{Arc, Mutex};
use cosmrs::AccountId;
use cosmwasm_std::HexBinary;
use k256::ecdsa::SigningKey;
use quartz_common::{
    contract::state::Config,
    enclave::{
        attestor::Attestor,
        server::{IntoServer, WsListenerConfig},
    },
};
use crate::proto::settlement_server::{Settlement, SettlementServer};

impl<A: Attestor> IntoServer for FairblockService<A> {
    type Server = SettlementServer<FairblockService<A>>;

    fn into_server(self) -> Self::Server {
        SettlementServer::new(self)
    }
}

pub type RawCipherText = HexBinary;

#[derive(Clone, Debug)]
pub struct FairblockOp<A: Attestor> {
    pub client: FairblockService<A>,
    pub config: WsListenerConfig,
}

#[derive(Clone, Debug)]
pub struct FairblockService<A: Attestor> {
    config: Config,
    contract: Arc<Mutex<Option<AccountId>>>,
    sk: Arc<Mutex<Option<SigningKey>>>,
    attestor: A,
}

impl<A> FairblockService<A>
where
    A: Attestor,
{
    pub fn new(
        config: Config,
        contract: Arc<Mutex<Option<AccountId>>>,
        sk: Arc<Mutex<Option<SigningKey>>>,
        attestor: A,
    ) -> Self {
        Self {
            config,
            contract,
            sk,
            attestor,
        }
    }
}

#[tonic::async_trait]
impl<A> Settlement for FairblockService<A>
where
    A: Attestor + Send + Sync + 'static,
{
}
