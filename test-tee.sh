#!/bin/bash

# Set up environment variables
export NODE_URL="http://127.0.0.1:26659"
export PATH=$PATH:/usr/local/go/bin
export LIBCLANG_PATH="/usr/lib/llvm-14/lib"

# Kill any running processes that might conflict
pkill -f "./target/release/quartz-app-fairblock-enclave"
pkill -f "fairyringd"

# Exit immediately on failure and clean up background processes
set -e  
trap 'kill $(jobs -p) &> /dev/null || true; exit 1' EXIT

# Start fairyring chain
cd ../fairyring
make install
sudo cp ~/go/bin/fairyringd /usr/local/go/bin/fairyringd
sudo rm -r ~/go/bin/fairyringd
make fresh-chain > fairyring_chain.log 2>&1 &

# Build CLI, enclave, and contracts
cd ../fairblock-cycles-quartz
cargo install --path crates/cli

sleep 5
cargo run -- contract build --contract-manifest "crates/contracts/tcbinfo/Cargo.toml"
RES=$(fairyringd --node="$NODE_URL" tx wasm store target/wasm32-unknown-unknown/release/quartz_tcbinfo.wasm --from star -y --output json --chain-id "fairyring" --gas-prices 0.0053ufairy --gas auto --gas-adjustment 1.3 --keyring-backend test)
TX_HASH=$(echo "$RES" | jq -r '.["txhash"]')
sleep 5

CERT=$(sed ':a;N;$!ba;s/\n/\\n/g' crates/contracts/tee-ra/data/root_ca.pem)
CODE_ID=1
fairyringd --node="$NODE_URL" tx wasm instantiate "$CODE_ID" "{\"root_cert\": \"$CERT\"}" --from "star" --label "tcbinfo" --chain-id "fairyring" --gas-prices 0.0053ufairy --gas auto --gas-adjustment 1.3 --keyring-backend test -y --no-admin --output json
sleep 5
TCB_CONTRACT=$(fairyringd --node="$NODE_URL" query wasm list-contract-by-code "$CODE_ID" --output json | jq -r '.contracts[0]')
echo "TCB Contract Address: $TCB_CONTRACT"

# Get TCB Info from Intel
export FMSPC="00606A000000"
HEADERS=$(wget -q -S -O - https://api.trustedservices.intel.com/sgx/certification/v4/tcb?fmspc="$FMSPC" 2>&1 >/dev/null)
TCB_INFO=$(wget -q -O - https://api.trustedservices.intel.com/sgx/certification/v4/tcb?fmspc="$FMSPC")
TCB_ISSUER_CERT=$(echo "$HEADERS" | grep 'TCB-Info-Issuer-Chain:' | sed 's/.*TCB-Info-Issuer-Chain: //' | perl -MURI::Escape -ne 'print uri_unescape($_)' | awk '/-----BEGIN CERTIFICATE-----/{flag=1; print; next} /-----END CERTIFICATE-----/{print; flag=0; exit} flag')
TCB_ISSUER_CERT=$(echo "$TCB_ISSUER_CERT" | sed ':a;N;$!ba;s/\n/\\n/g')

echo "TCB_INFO: $TCB_INFO"
echo "TCB_ISSUER_CERT: $TCB_ISSUER_CERT"

sleep 5
fairyringd --node="$NODE_URL" tx wasm execute "$TCB_CONTRACT" "{\"tcb_info\": $(echo "$TCB_INFO" | jq -Rs .), \"certificate\": \"$TCB_ISSUER_CERT\"}" --from star --chain-id fairyring --fees 1000ufairy --gas-adjustment 1.5 --gas auto --keyring-backend test -y 

# Query the TCB info from contract
sleep 5
fairyringd --node="$NODE_URL" query wasm contract-state smart "$TCB_CONTRACT" "{\"get_tcb_info\": {\"fmspc\": \"${FMSPC}\"}}"

# Build and Deploy DCAP Verifier Contract
cargo run -- contract build --contract-manifest "crates/contracts/dcap-verifier/Cargo.toml"
wasm-opt -Oz target/wasm32-unknown-unknown/release/quartz_dcap_verifier.wasm -o target/wasm32-unknown-unknown/release/quartz_dcap_verifier.optimized.wasm --all-features
sleep 5
RES=$(fairyringd --node="$NODE_URL" tx wasm store target/wasm32-unknown-unknown/release/quartz_dcap_verifier.optimized.wasm --from star -y --output json --chain-id "fairyring" --gas-prices 0.0025ufairy --gas auto --keyring-backend test --gas-adjustment 1.3)
TX_HASH=$(echo "$RES" | jq -r '.["txhash"]')
sleep 5
CODE_ID=2
fairyringd --node="$NODE_URL" tx wasm instantiate "$CODE_ID" null --from "star" --label "dcap-verifier" --chain-id "fairyring" --gas-prices 0.0025ufairy --gas auto --gas-adjustment 1.3 -y --no-admin --keyring-backend test --output json
sleep 5
DCAP_CONTRACT=$(fairyringd --node="$NODE_URL" query wasm list-contract-by-code "$CODE_ID" --output json | jq -r '.contracts[0]')
echo "DCAP Contract Address: $DCAP_CONTRACT"

# Start Enclave
export ADMIN_SK="b1b38cfc3ce43d409acaabbbce6c6ae13c6c2a164311e6df0571a380a7439a8e"
cd examples/fairblock
cp quartz.neutron_pion-1.toml quartz.toml
quartz enclave build
quartz enclave start --fmspc "$FMSPC" --tcbinfo-contract "$TCB_CONTRACT" --dcap-verifier-contract "$DCAP_CONTRACT" --unsafe-trust-latest > enclave_output.log 2>&1 &
sleep 10

# Build and Deploy Fairblock Contract
quartz contract build --contract-manifest "contracts/Cargo.toml"
sleep 3
output=$(quartz contract deploy --contract-manifest "contracts/Cargo.toml" --init-msg '{}')
json_output=$(echo "$output" | grep -o '{.*}')
contract_address=$(echo "$json_output" | jq -r '.ContractDeploy.contract_addr')

if [[ -z "$contract_address" || "$contract_address" == "null" ]]; then
    echo "Error: Contract address not found in the output." >&2
    exit 1
fi

echo "Contract Address: $contract_address"
sleep 4

# Handshake and Retrieve SK Value
quartz handshake --contract "$contract_address"
sleep 3
cleaned_log=$(sed -E 's/\x1b\[[0-9;]*m//g' enclave_output.log)
sk_value=$(echo "$cleaned_log" | grep -oP '(?<=sk:")[a-f0-9]+"')
sk_value=$(echo "$sk_value" | tr -d '"')

if [[ -z "$sk_value" ]]; then
    echo "Error: Could not extract sk from the log." >&2
    exit 1
fi
echo "SK Value: $sk_value"

# Test Key Sharing
cd ../../../fairyring/test
encrypted_share=$(cargo run --release "$sk_value")
fairyringd tx keyshare create-latest-pubkey a83ec58f7772aee8a11029da99b4af74f19ef9f9b95559dfa32293115d5089c565d193046ef299e628703844f00f0c5b b584990d7022c6989633b0d443ffc5fc1128b4107cac25904d526d12536153c34349e5f3657870a498ccf6f78a858085 1 "[{\"data\":\"$encrypted_share\",\"validator\":\"fairy1vghpa0tuzfza97cwyc085zxuhsyvy3jtgry7vv\"}]" --from fairy1vghpa0tuzfza97cwyc085zxuhsyvy3jtgry7vv --chain-id fairyring --fees 5000ufairy --node "$NODE_URL" --keyring-backend test -y
sleep 5

# Request Decryption Key
for i in {1..5}; do
    fairyringd tx pep request-general-identity "30s" test-$i --from star --node "$NODE_URL" --chain-id fairyring --fees 300ufairy --keyring-backend test -y
    sleep 5
    fairyringd tx pep request-general-decryption-key "fairy1vghpa0tuzfza97cwyc085zxuhsyvy3jtgry7vv/test-$i" --from star --node "$NODE_URL" --chain-id fairyring --fees 300ufairy --keyring-backend test -y
    sleep 5
done
