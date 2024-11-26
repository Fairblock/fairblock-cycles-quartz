#!/bin/bash

set -e  # Exit immediately if a command fails
trap 'kill $(jobs -p) &> /dev/null || true; exit 1' EXIT  # Stop background processes on exit

export PATH=$PATH:$(go env GOPATH)/bin

# Start fairyring
cd ../fairyring

make install
make fresh-chain > fairyring_chain.log 2>&1 &

# Building the CLI, enclave, and contracts
cd ../fairblock-cycles-quartz
cargo install --path crates/cli

cd examples/fairblock

quartz --mock-sgx enclave build
quartz --mock-sgx contract build --contract-manifest "contracts/Cargo.toml"

# Start the enclave
export ADMIN_SK=b1b38cfc3ce43d409acaabbbce6c6ae13c6c2a164311e6df0571a380a7439a8e
quartz --mock-sgx enclave start > enclave_output.log 2>&1 &

# Deploy the contract and perform handshake
output=$(quartz --mock-sgx contract deploy --contract-manifest "contracts/Cargo.toml" --init-msg '{}')

# Extract JSON part from the output
json_output=$(echo "$output" | grep -o '{.*}')

# Parse the contract address from the JSON
contract_address=$(echo "$json_output" | jq -r '.ContractDeploy.contract_addr')

# Check if the contract address was extracted successfully
if [[ -z "$contract_address" || "$contract_address" == "null" ]]; then
    echo "Error: Contract address not found in the output." >&2
    exit 1
fi

echo "Contract Address: $contract_address"

# Use the extracted contract address
quartz --mock-sgx handshake --contract "$contract_address"

cleaned_log=$(cat enclave_output.log | sed -E 's/\x1b\[[0-9;]*m//g')
sk_value=$(echo "$cleaned_log" | grep -oP '(?<=sk:")[a-f0-9]+"')
sk_value=$(echo "$sk_value" | tr -d '"')
if [[ -z "$sk_value" ]]; then
    echo "Error: Could not extract sk from the log." >&2
    exit 1
fi
echo "$sk_value"


# Test execution
cd ../../../test

encrypted_share=$(cargo run "$sk_value")
i=1
fairyringd tx keyshare create-latest-pubkey a83ec58f7772aee8a11029da99b4af74f19ef9f9b95559dfa32293115d5089c565d193046ef299e628703844f00f0c5b b584990d7022c6989633b0d443ffc5fc1128b4107cac25904d526d12536153c34349e5f3657870a498ccf6f78a858085 1 "[{\"data\":\"$encrypted_share\",\"validator\":\"fairy1vghpa0tuzfza97cwyc085zxuhsyvy3jtgry7vv\"}]" --from fairy1vghpa0tuzfza97cwyc085zxuhsyvy3jtgry7vv --chain-id fairyring --fees 5000ufairy --node http://127.0.0.1:26659 -y
sleep 5
fairyringd tx pep request-general-identity "30s" test-$i --from star --node http://127.0.0.1:26659 --chain-id fairyring --fees 300ufairy -y
sleep 5
fairyringd tx pep request-general-decryption-key fairy1vghpa0tuzfza97cwyc085zxuhsyvy3jtgry7vv/test-$i --from star --node http://127.0.0.1:26659 --chain-id fairyring --fees 300ufairy -y

trap - EXIT  # Remove trap to avoid cleanup at the end of successful execution
