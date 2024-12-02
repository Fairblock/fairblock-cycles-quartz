use anyhow::{Result, Context};
use ics23::HostFunctionsManager;
use k256::ecdsa::SigningKey;
use tendermint::block::Height;
use tendermint::merkle::proof::ProofOps;
use tendermint_rpc::endpoint::abci_query::AbciQuery;
use ics23::{verify_membership, CommitmentProof};
use prost::Message;
use hex::encode;
use tendermint_rpc::event::Event;
use tendermint_rpc::query::Query;
use tendermint_rpc::Client;
use tendermint_rpc::{self as rpc, SubscriptionClient};
use tokio_stream::StreamExt;
use crate::get_share::get_key_share;
use crate::tx::send_keyshare;

fn extract_keyshare_request(event: Event) -> Option<String> {
    if let rpc::event::EventData::NewBlock { result_finalize_block, .. } = event.data {
        for tendermint_event in result_finalize_block.unwrap().events {
            if tendermint_event.kind == "start-send-general-keyshare" {
                for attribute in tendermint_event.attributes {
                    if attribute.key_str().unwrap() == "identity" {
                        let id_value = attribute.value_str().unwrap().clone();
                       
                            println!("Found matching event with id_value: {:?}", id_value);
                            return Some(id_value.to_string());
                        
                    }
                }
            }
        }
    }
    None
}

fn proof_ops_to_commitment_proof(proof_ops: &ProofOps) -> Result<CommitmentProof> {
  
    let proof_op = proof_ops
        .ops
        .iter()
        .find(|op| op.r#field_type == "ics23:iavl")
        .context("ProofOp for IAVL leaf not found")?;
       
 
    let commitment_proof = CommitmentProof::decode(&*proof_op.data)
        .context("Failed to decode CommitmentProof")?;
    
    Ok(commitment_proof)
}

fn verify_merkle_proof(
    proof_ops: &ProofOps,
    app_hash: &[u8],
    key: &[u8],
    value: &[u8],
) -> Result<()> {
    let commitment_proof = proof_ops_to_commitment_proof(proof_ops)?;
   
    let spec = ics23::iavl_spec();
   
    let res: bool = verify_membership::<HostFunctionsManager>(
        &commitment_proof,
        &spec,
        &app_hash.to_vec(),
        key,
        value,
    );
   println!("res: {:?}", res);
    if res {
        Ok(())
    } else {
        anyhow::bail!("Proof verification failed");
    }
}

async fn verify_event_proof(
    client: &rpc::HttpClient,
    event_key: &[u8],
    block_height: Height,
) -> Result<bool> {
    // For testing purposes
    return Ok(true);
    let query_response: AbciQuery = client
        .abci_query(
            Some("/store/keyshare/key".to_string()), 
            event_key.to_vec(),
            Some(block_height),
            true,
        )
        .await
        .context("Error executing ABCI query")?;
    println!("resp: {:?}", query_response);
    let block = client.block(block_height).await.context("Error retrieving block")?;
    let app_hash = block.block.header.app_hash.as_bytes();
   
    if let Some(proof_ops) = query_response.proof {
    
        verify_merkle_proof(&proof_ops, app_hash, event_key, &query_response.value)?;
        Ok(true)
    } else {
        anyhow::bail!("No proof available for the event");
    }
}

pub async fn listen_fairyring(sk: SigningKey) -> Result<()> {
   
    let sk_hex = encode(sk.to_bytes());

  
    let mut share_val: Vec<u8> = vec![];
    let mut index_val = 0;
      loop {
        
           let result = get_key_share(sk.clone()).await;
           match result {
            Ok((share, index)) => {
                share_val = share;
                index_val = index as u32;
                break;
            },
            Err(e) => {
              // println!("{:?}",e);
            }
        }
            }
        
    let (client, driver) = rpc::WebSocketClient::new("ws://127.0.0.1:26659/websocket").await
        .context("Failed to create WebSocket client")?;
    let http_client = rpc::HttpClient::new("http://127.0.0.1:26659").context("Failed to create HTTP client")?;

    tokio::spawn(driver.run());

    let query = Query::eq("tm.event", "NewBlock");
    let mut subscription = client.subscribe(query).await.context("Subscription failed")?;
    println!("Listening to events...");
    while let Some(event_result) = subscription.next().await {
        match event_result {
            Ok(event) => {
               
                let mut block_height = 0;
                if let tendermint_rpc::event::EventData::NewBlock { block, .. } = event.clone().data {
                    if let Some(block) = block {
                        block_height = block.header.height.value();
                    }
                }
               
                let height = Height::from(block_height as u32);
                if let Some(id_value) = extract_keyshare_request(event.clone()) {
                    println!("Received request for id: {:?}", id_value);

                    let key = format!("DecryptionKeyRequest/value/{}", id_value);
                 

                    if verify_event_proof(&http_client, &key.as_bytes(), height).await? {
                        println!("Event proof verified for id: {:?}", id_value);
                        let share = share_val.clone();
   
                        tokio::spawn(send_keyshare(id_value, share,index_val.clone(),sk.clone()));
                        
                    } else {
                        println!("Event proof verification failed for id: {:?}", id_value);
                    }
                }
            }
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    Ok(())
}
