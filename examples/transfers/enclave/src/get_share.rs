use reqwest::Client;
use eyre::{Result, eyre};
use serde::Deserialize;
use ecies::{decrypt, encrypt};
use k256::ecdsa::{SigningKey, VerifyingKey};
use tonic::Status;
use base64::decode;
use tracing::info;

#[derive(Debug, Deserialize)]
struct PublicKey {
    active_pubkey: PubkeyInfo,
    queued_pubkey: Option<PubkeyInfo>,
}

#[derive(Debug, Deserialize)]
struct PubkeyInfo {
    encrypted_keyshares: Vec<EncryptedKeyshare>,
    expiry: String,
}

#[derive(Debug, Deserialize)]
struct EncryptedKeyshare {
    validator: String,
    data: String,
}

async fn get_active_pubkey(client: &Client, base_url: &str) -> Result<PublicKey> {
    let url = format!("{}/fairyring/keyshare/pubkey", base_url);
    let response = client.get(&url).send().await?.json::<PublicKey>().await?;
    Ok(response)
}

fn decrypt_share(sk: SigningKey, data: &[u8]) -> Result<Vec<u8>> {
    let o =
    decrypt(&sk.to_bytes(), data).map_err(|e| Status::invalid_argument(e.to_string()))?;
    Ok(o) 
}


pub async fn get_key_share(
    sk: SigningKey,
) -> Result<(Vec<u8>,u64)> {
    
    let base_url = "http://127.0.0.1:1317";
    let client = Client::new();
  
   
    let pub_key = get_active_pubkey(&client, base_url).await?;

   
    let target_enc_keyshare_list = &pub_key.active_pubkey.encrypted_keyshares;
   
    if target_enc_keyshare_list.is_empty() {
        return Err(eyre!("Encrypted shares array for target round is empty"));
    }

    for (index, enc_share) in target_enc_keyshare_list.iter().enumerate() {
        if enc_share.validator == "fairy1vghpa0tuzfza97cwyc085zxuhsyvy3jtgry7vv" {
           
            println!("Keyshare found...");
            let decoded_bytes = decode(enc_share.data.clone()).expect("Failed to decode Base64");
            println!("Decrypting the keyshare");
            let decrypted_bytes = decrypt_share(sk, &decoded_bytes)?;
            println!("Keyshare: {:?}", decrypted_bytes);
            return Ok((decrypted_bytes, index as u64 + 1));
        }
    }

    Err(eyre!("Encrypted share for your validator not found"))
}
