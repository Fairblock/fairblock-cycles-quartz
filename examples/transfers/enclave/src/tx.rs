use cosmos_sdk_proto::cosmos::tx::v1beta1::{BroadcastMode, TxBody};
use cosmos_sdk_proto::cosmos::tx::v1beta1::{service_client::ServiceClient as TxServiceClient};
use cosmrs::tendermint::chain;
use cosmrs::{tx, Any};
use cosmrs::{
    tx::{Fee, SignDoc, SignerInfo},
    AccountId, Coin,
};

use k256::ecdsa::{SigningKey, Signature, signature::Signer};
use cosmrs::crypto::{secp256k1, PublicKey};

use reqwest::Client;
use prost::Message;
use eyre::Result;
use hex::{self, FromHex};
use serde_json::Value;
use sha2::{Sha256, Digest};
use tokio::time::{timeout, Duration};
use tendermint::chain::Id as ChainId;
use crate::extract::{self, extract};

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitGeneralKeyshare {
    #[prost(string, tag = "1")]
    pub creator: String,
    #[prost(string, tag = "2")]
    pub id_type: String,
    #[prost(string, tag = "3")]
    pub id_value: String,
    #[prost(string, tag = "4")]
    pub keyshare: String,
    #[prost(uint64, tag = "5")]
    pub keyshare_index: u64,
    #[prost(uint64, tag = "6")]
    pub received_timestamp: u64,
    #[prost(uint64, tag = "7")]
    pub received_block_height: u64,
    #[prost(string, tag = "8")]
    pub signature: String,
}

pub async fn send_keyshare(
    identity: String,
    share: Vec<u8>,
    index: u32,
    k256_sk: SigningKey,
) -> Result<()> {
    // Step 1: Derive the key share using the `extract` function
    let extracted_key = extract(share, identity.as_bytes().to_vec(), index).unwrap();
    println!("Derived General Key Share: {:?}", extracted_key);

    // Step 2: Hash the fields for `k256` signing
    let mut hasher = Sha256::new();
    hasher.update(identity.as_bytes()); // id_value
    hasher.update(&extracted_key); // key_share
    hasher.update(index.to_le_bytes()); // key_share_index
    let k256_msg_hash = hasher.finalize();
    println!("message hash: {:?}", k256_msg_hash);
    // Step 3: Sign the hash with the `k256` key
    let k256_signature: Signature = k256_sk.sign(&k256_msg_hash);

    // Convert the DER signature to raw format
    let r = k256_signature.r();
    let s = k256_signature.s();

// Convert r and s to 32-byte big-endian representations
    let mut raw_signature = Vec::with_capacity(64);
    raw_signature.extend_from_slice(&r.to_bytes());
    raw_signature.extend_from_slice(&s.to_bytes());
    let k256_signature_hex = hex::encode(raw_signature);
    println!("k256 Signature: {}", k256_signature_hex);

    // Step 4: Cosmos key setup
    let mut sender_private_key = secp256k1::SigningKey::random();
    let private_key_hex = "b1b38cfc3ce43d409acaabbbce6c6ae13c6c2a164311e6df0571a380a7439a8e";

    match Vec::from_hex(private_key_hex) {
        Ok(private_key_bytes) => match secp256k1::SigningKey::from_slice(&private_key_bytes) {
            Ok(signing_key) => sender_private_key = signing_key,
            Err(e) => println!("Error creating signing key: {}", e),
        },
        Err(e) => println!("Error decoding hex: {}", e),
    }

    let sender_public_key = sender_private_key.public_key();
    let acc_address = sender_public_key.account_id("fairy").unwrap().to_string();
    println!("account: {:?}", acc_address);
    // Step 5: Create the MsgSubmitGeneralKeyshare message
    let msg = MsgSubmitGeneralKeyshare {
        creator: acc_address.clone(),
        id_type: "private-gov-identity".to_string(),
        id_value: identity.clone(),
        keyshare: hex::encode(&extracted_key),
        keyshare_index: index as u64,
        received_timestamp: 4294967294,
        received_block_height: 4294967294,
        signature:k256_signature_hex
    };

    // Step 6: Fetch account information
    let base_url = "http://127.0.0.1:1317/cosmos/auth/v1beta1/accounts";
    let url = format!("{}/{}", base_url, acc_address);
    let client = Client::new();
    let response: Value = client.get(&url).send().await?.json().await?;
    let account_number: u64 = response["account"]["account_number"]
        .as_str()
        .unwrap()
        .parse()?;
    let sequence_number: u64 = response["account"]["sequence"]
        .as_str()
        .unwrap()
        .parse()?;

    // Step 7: Construct the transaction body
    let chain_id: chain::Id = "fairyring".parse()?;
    let gas = 100_000u64;
    let timeout_height = 4294967294u32;

    let tx_body = tx::Body::new(
        vec![Any {
            type_url: "/fairyring.keyshare.MsgSubmitGeneralKeyshare".to_owned(),
            value: msg.encode_to_vec(),
        }],
        "", 
        timeout_height,
    );

    let signer_info = SignerInfo::single_direct(Some(sender_public_key), sequence_number);
    let auth_info = signer_info.auth_info(Fee::from_amount_and_gas(
        Coin {
            amount: 100u128,
            denom: "ufairy".parse()?,
        },
        gas,
    ));

    let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, account_number)?;
    let tx_signed = sign_doc.sign(&sender_private_key)?;

    let tx_bytes = tx_signed.to_bytes()?;
    let tx_hex = hex::encode(tx_bytes);

    // Step 8: Broadcast the transaction
    let response = timeout(Duration::from_secs(10), async {
        Client::new()
            .post(&format!(
                "http://127.0.0.1:26659/broadcast_tx_sync?tx=0x{}",
                tx_hex
            ))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await
    })
    .await??;

    println!("Broadcast response: {:?}", response);
    Ok(())
}
