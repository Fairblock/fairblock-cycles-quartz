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

impl<A: Attestor> IntoServer for TransfersService<A> {
    type Server = SettlementServer<TransfersService<A>>;

    fn into_server(self) -> Self::Server {
        SettlementServer::new(self)
    }
}

pub type RawCipherText = HexBinary;

#[derive(Clone, Debug)]
pub struct TransfersOp<A: Attestor> {
    pub client: TransfersService<A>,
    pub config: WsListenerConfig,
}

#[derive(Clone, Debug)]
pub struct TransfersService<A: Attestor> {
    config: Config,
    contract: Arc<Mutex<Option<AccountId>>>,
    sk: Arc<Mutex<Option<SigningKey>>>,
    attestor: A,
}

impl<A> TransfersService<A>
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
impl<A> Settlement for TransfersService<A>
where
    A: Attestor + Send + Sync + 'static,
{
}
