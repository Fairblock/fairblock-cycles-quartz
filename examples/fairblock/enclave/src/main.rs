#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(
    clippy::checked_conversions,
    clippy::panic,
    clippy::panic_in_result_fn,
    trivial_casts,
    trivial_numeric_casts,
    rust_2018_idioms,
    unused_lifetimes,
    unused_import_braces,
    unused_qualifications
)]

pub mod cli;
pub mod extract;
pub mod fairy_listener;
pub mod get_share;
pub mod proto;
pub mod fairblock_server;
pub mod tx;
pub mod wslistener;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use clap::Parser;
use cli::Cli;
use fairy_listener::listen_fairyring;

use quartz_common::{
    contract::state::{Config, LightClientOpts},
    enclave::{
        attestor::{self, Attestor},
        server::{QuartzServer, WsListenerConfig},
    },
};

use fairblock_server::FairblockService;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let admin_sk = std::env::var("ADMIN_SK")
        .map_err(|_| anyhow::anyhow!("Admin secret key not found in env vars"))?;

    let light_client_opts = LightClientOpts::new(
        args.chain_id.clone(),
        args.trusted_height.into(),
        Vec::from(args.trusted_hash)
            .try_into()
            .expect("invalid trusted hash"),
        (
            args.trust_threshold.numerator(),
            args.trust_threshold.denominator(),
        ),
        args.trusting_period,
        args.max_clock_drift,
        args.max_block_lag,
    )?;

    #[cfg(not(feature = "mock-sgx"))]
    let attestor = attestor::DcapAttestor {
        fmspc: args.fmspc.expect("FMSPC is required for DCAP"),
    };

    #[cfg(feature = "mock-sgx")]
    let attestor = attestor::MockAttestor::default();

    let config = Config::new(
        attestor.mr_enclave()?,
        Duration::from_secs(30 * 24 * 60),
        light_client_opts,
        args.tcbinfo_contract.map(|c| c.to_string()),
        args.dcap_verifier_contract.map(|c| c.to_string()),
    );

    let ws_config = WsListenerConfig {
        node_url: args.node_url,
        ws_url: args.ws_url,
        grpc_url: args.grpc_url,
        tx_sender: args.tx_sender,
        trusted_hash: args.trusted_hash,
        trusted_height: args.trusted_height,
        chain_id: args.chain_id,
        admin_sk,
    };

    let contract = Arc::new(Mutex::new(None));
    let sk = Arc::new(Mutex::new(None));

    let sk_clone1 = sk.clone();
    let sk_clone2 = sk.clone();
    tokio::spawn(async move {
        let _ = QuartzServer::new(
            config.clone(),
            sk_clone1,
            contract.clone(),
            attestor.clone(),
            ws_config.clone(),
        )
        .add_service(FairblockService::new(config, contract, sk_clone2, attestor))
        .serve(args.rpc_addr)
        .await;
    });

    loop {
        let maybe_key = match sk.lock() {
            Ok(guard) => guard.clone(),
            Err(e) => {
                eprintln!("Mutex lock failed: {:?}", e);
                None
            }
        };

        if let Some(signing_key) = maybe_key {
            listen_fairyring(signing_key).await?;
            break;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}
