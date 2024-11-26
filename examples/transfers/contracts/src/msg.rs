use cosmwasm_schema::cw_serde;
use quartz_common::contract::{
    msg::execute::attested::{RawAttested, RawAttestedMsgSansHandler, RawDefaultAttestation},
    prelude::*,
};
pub type AttestedMsg<M, RA = RawDefaultAttestation> = RawAttested<RawAttestedMsgSansHandler<M>, RA>;

#[cw_serde]
pub struct InstantiateMsg<RA = RawDefaultAttestation> {
    pub quartz: QuartzInstantiateMsg<RA>,
}

#[cw_serde]
pub enum QueryMsg {
    GetPublicKeys {}, // Query for registered public keys
}

#[cw_serde]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg<RA = RawDefaultAttestation> {
    // Quartz message for initializing sessions and setting public keys
    Quartz(QuartzExecuteMsg<RA>),
}

pub mod execute {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::HexBinary;
    use quartz_common::contract::{msg::execute::attested::HasUserData, state::UserData};
    use sha2::{Digest, Sha256};

    #[cw_serde]
    pub struct UpdateMsg {
        pub ciphertext: HexBinary,
        pub quantity: u32,
    }

    impl HasUserData for UpdateMsg {
        fn user_data(&self) -> UserData {
            let mut hasher = Sha256::new();
            hasher.update(serde_json::to_string(&self).expect("infallible serializer"));
            let digest: [u8; 32] = hasher.finalize().into();

            let mut user_data = [0u8; 64];
            user_data[0..32].copy_from_slice(&digest);
            user_data
        }
    }

    #[cw_serde]
    pub struct QueryResponseMsg {
        pub address: String,
        pub encrypted_bal: HexBinary,
    }

    impl HasUserData for QueryResponseMsg {
        fn user_data(&self) -> UserData {
            let mut hasher = Sha256::new();
            hasher.update(serde_json::to_string(&self).expect("infallible serializer"));
            let digest: [u8; 32] = hasher.finalize().into();

            let mut user_data = [0u8; 64];
            user_data[0..32].copy_from_slice(&digest);
            user_data
        }
    }
}
