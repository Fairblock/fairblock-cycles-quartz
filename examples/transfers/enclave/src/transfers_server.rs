use std::{
    collections::{btree_map::Entry, BTreeMap},
    sync::{Arc, Mutex},
};

use cosmrs::AccountId;
use cosmwasm_std::{Addr, HexBinary, Uint128};
use ecies::{decrypt, encrypt};
use k256::ecdsa::{SigningKey, VerifyingKey};
use quartz_common::{
    contract::{
        msg::execute::attested::{HasUserData, RawAttested},
        state::{Config, UserData},
    },
    enclave::{
        attestor::Attestor,
        server::{IntoServer, ProofOfPublication, WsListenerConfig},
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::mpsc::Sender;
use tonic::{Request, Response, Result as TonicResult, Status};


use crate::{
    proto::{
        settlement_server::{Settlement, SettlementServer},
        QueryRequest, QueryResponse, UpdateRequest, UpdateResponse,
    },
  
};

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

