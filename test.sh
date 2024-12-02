#!/bin/bash

# Terminate any conflicting processes
pkill -f "./target/release/quartz-app-fairblock-enclave"
pkill -f "fairyringd"

# Set up environment and error handling
set -e  
trap 'kill $(jobs -p) &> /dev/null || true; exit 1' EXIT
export PATH=$PATH:/usr/local/go/bin

# Start fairyring
cd ../fairyring
make install
sudo cp ~/go/bin/fairyringd /usr/local/go/bin/fairyringd
make fresh-chain > fairyring_chain.log 2>&1 &
sleep 5

# Build CLI, enclave, and contracts
cd ../fairblock-cycles-quartz
cargo install --path crates/cli

cd examples/fairblock
quartz --mock-sgx enclave build
sleep 3
quartz --mock-sgx contract build --contract-manifest "contracts/Cargo.toml"
sleep 3

# Start the enclave
export ADMIN_SK="b1b38cfc3ce43d409acaabbbce6c6ae13c6c2a164311e6df0571a380a7439a8e"
quartz --mock-sgx enclave start > enclave_output.log 2>&1 &
sleep 5

# Deploy the contract and perform handshake
output=$(quartz --mock-sgx contract deploy --contract-manifest "contracts/Cargo.toml" --init-msg '{}')
json_output=$(echo "$output" | grep -o '{.*}')
contract_address=$(echo "$json_output" | jq -r '.ContractDeploy.contract_addr')

# Validate contract address extraction
if [[ -z "$contract_address" || "$contract_address" == "null" ]]; then
    echo "Error: Contract address not found in the output." >&2
    exit 1
fi

echo "Contract Address: $contract_address"

# Handshake using the extracted PK
output=$(quartz --mock-sgx handshake --contract "$contract_address")

# Extract the pub_key value from the output
pub_key=$(echo "$output" | grep -oP '(?<="pub_key":")[^"]+')

# Print the extracted pub_key
if [[ -n $pub_key ]]; then
    echo "Extracted pub_key: $pub_key"
else
    echo "pub_key not found in the command output."
fi


# Test execution
cd ../../../fairyring/test
encrypted_share=$(cargo run --release "$pub_key")

fairyringd tx keyshare create-latest-pubkey \
    a83ec58f7772aee8a11029da99b4af74f19ef9f9b95559dfa32293115d5089c565d193046ef299e628703844f00f0c5b \
    b584990d7022c6989633b0d443ffc5fc1128b4107cac25904d526d12536153c34349e5f3657870a498ccf6f78a858085 \
    1 "[{\"data\":\"$encrypted_share\",\"validator\":\"fairy1vghpa0tuzfza97cwyc085zxuhsyvy3jtgry7vv\"}]" \
    --from fairy1vghpa0tuzfza97cwyc085zxuhsyvy3jtgry7vv --chain-id fairyring --fees 5000ufairy \
    --node http://127.0.0.1:26659 --keyring-backend test -y
sleep 5

# Request Decryption Key
for i in {1..5}; do
    fairyringd tx pep request-general-identity "30s" test-$i \
        --from star --node http://127.0.0.1:26659 --chain-id fairyring \
        --fees 300ufairy --keyring-backend test -y
    sleep 5

    fairyringd tx pep request-general-decryption-key \
        fairy1vghpa0tuzfza97cwyc085zxuhsyvy3jtgry7vv/test-$i \
        --from star --node http://127.0.0.1:26659 --chain-id fairyring \
        --fees 300ufairy --keyring-backend test -y
    sleep 5
done
