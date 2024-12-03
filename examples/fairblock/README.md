# Fairblock-Cycles-Quartz

## Why Fairblock-Cycles-Quartz?

Fairblock-Cycles-Quartz is designed to address the need for secure and reliable validator operations in Fairyring. Validators play an important role in maintaining the integrity of our framework, but the potential for collusion among them can compromise the system. This framework leverages Cycles-Quartz libraries to implement our validator functionality within a TEE, ensuring that sensitive data is stored securely.

The risks of validator collusion are thoroughly explored in our [Multimodal Cryptography Series – Accountable MPC + TEE](https://hackmd.io/@Fairblock/rkSiU78TR) article, which highlights how collusion can lead to unwanted outcomes such as decryption key leakage before the designated decryption time. Fairblock-Cycles-Quartz ensures that validators remain unaware of their shares and hence, cannot misuse them. By running critical processes inside a TEE, the framework guarantees that storing shares and key extraction is isolated from validators. This approach not only secures sensitive data but also provides on-chain attestation to verify that the messages sent by validators actually originate from inside valid TEEs.



## Overview

The validator registration process for Fairblock-Cycles-Quartz relies on a CosmWasm contract to manage TEEs within the Fairyring network. This process begins with a handshake between the contract and an enclave using the Quartz client. Upon successful completion, the contract stores the public key of the enclave in a list representing the registered TEEs. This list can be used to verify the messages originating from inside the enclaves.

Once the validators are registered, their public keys are retrieved from the contract. Each validator’s share is encrypted using their registered public key, and the encrypted shares are sent on-chain. The enclaves fetch these encrypted shares and perform decryption within the TEE using the corresponding secret key. This ensures that the validators themselves remain unaware of their shares, preventing them from colluding with each other.

The enclaves also monitor the blockchain for decryption key requests. Upon receiving a request, the enclave validates it against the chain state using Tendermint’s `abci_query` mechanism. If the validation is successful, the key share is securely extracted and signed using the enclave’s secret key, which corresponds to the public key stored on-chain. This signed share is then submitted back to Fairyring. This signature is verified on chain using the list of the registered PKs from the contract to confirm that the message originates from a valid TEE. After successful verification, the extracted key can be used for key aggregation.

The diagram below illustrates the overall process:
![Fairblock-Cycles-Quartz](./cycles.png)


## Implementation Details

This implementation builds upon the `Transfers` example from Cycles-Quartz, with modifications to the Quartz cli to enable contract deployment and interaction with Fairyring.

The CosmWasm contract is modified to store a list of PKs from validated enclaves through the handshake process. On the enclave side, additional functionalities were implemented to support validator operations for Fairyring. Specifically, the enclave starts by waiting for the handshake to complete and for the SK to be set. Once the SK is established, it fetches and decrypts its share using that SK. Following this, the enclave begins listening for decryption key requests from Fairyring. After verifying the requests, it extracts the key share and submits it on-chain.



## Testing

Testing for this framework involves two main scripts. The first, `test.sh`, is used for end-to-end testing without TEE integration. The second script, `test-tee.sh`, includes additional steps to deploy and configure the TCB and DCAP contracts on Fairyring, as well as performing the end-to-end test with the TEE enabled. 

The testing process involves several steps:
- Starting the Fairyring network.
- Building and starting the enclave.
- Deploying the CosmWasm contract and performing the handshake with the enclave.
- Encrypting the share using the enclave’s public key.
- Sending the encrypted share on-chain.
- Submitting a decryption key request to trigger the enclave’s process.

Note that for the end-to-end tests, the Fairyring source code (`abci-query` branch) is required to be cloned in the same directory where Fairblock-Cycles-Quartz is located.
Logs for the chain and enclave operations are stored in `fairyring/fairyring_chain.log` and `examples/fairblock/enclave_output.log`, respectively.



## Performance Analysis

The table below compares the average runtime for different operations executed in non-TEE and TEE environments. The overhead percentage reflects the additional runtime cost introduced by TEE integration. For each case, we ran the code `100` times and averaged the runtime. All tests were conducted on a Microsoft Azure server with the following specifications: **Standard DC4s v3 instance**, with **4 vCPUs** and **32 GiB of RAM**.


| Case                        | No TEE Average (ms) | TEE Average (ms) | Overhead (%)            |
|-----------------------------|----------------------|-------------------|-------------------------|
| Key Extraction              | 1.5015              | 1.5285           | +1.80%                 |
| Signing & Sending on Chain  | 81.8016             | 89.7137          | +9.68%                 |
| Get Share                   | 40.9942             | 44.2842          | +8.02%                 |

The results indicate a small overhead of approximately **1.80%** for key extraction, suggesting that the added security of the TEE has minimal impact on performance for this operation. Signing and sending transactions on-chain shows a higher overhead of **9.68%**. And the process of fetching and decrypting shares incurs an **8.02%** overhead.

These results demonstrate that while TEE integration does introduce some runtime cost, the overhead is limited to a maximum of **9.68%**, ensuring that the framework remains efficient for critical operations like extracting keys, signing, decrypting shares, and interacting with the blockchain.
