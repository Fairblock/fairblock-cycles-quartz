use std::{collections::BTreeMap, str::FromStr};

use anyhow::{anyhow, Error, Result};
use cosmrs::{tendermint::chain::Id as ChainId, AccountId};
use cosmwasm_std::{Addr, HexBinary};
use cw_client::{CwClient, GrpcClient};
use futures_util::StreamExt;
use quartz_common::{
    contract::msg::execute::attested::{RawAttested, RawAttestedMsgSansHandler},
    enclave::{
        attestor::Attestor,
        server::{WebSocketHandler, WsListenerConfig},
    },
};
use quartz_tm_prover::{config::Config as TmProverConfig, prover::prove};
use serde::Deserialize;
use serde_json::json;
use tendermint_rpc::{event::Event, query::EventType, SubscriptionClient, WebSocketClient};
use tonic::Request;
use tracing::info;
use transfers_contract::msg::{
    execute::{QueryResponseMsg,  UpdateMsg},
    AttestedMsg, ExecuteMsg
  
};

use crate::{
    proto::{settlement_server::Settlement, QueryRequest, UpdateRequest},
    transfers_server::{
       TransfersOp, TransfersService,
    },
};


// TODO: Need to prevent listener from taking actions until handshake is completed
#[async_trait::async_trait]
impl<A: Attestor + Clone> WebSocketHandler for TransfersService<A> {
    async fn handle(&self, event: Event, config: WsListenerConfig) -> Result<()> {

        Ok(())
    }
}

#[tonic::async_trait]
pub trait WsListener: Send + Sync + 'static {
   
}

#[async_trait::async_trait]
impl<A> WsListener for TransfersService<A>
where
    A: Attestor,
    A::RawAttestation: for<'de> Deserialize<'de> + Send,
{
  
}



