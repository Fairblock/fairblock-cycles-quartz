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



## 4. Performance Analysis

The performance of cryptographic operations was analyzed under environments with and without TEE integration. Specifically, we evaluated the average runtime of operations related to key extraction, signing and sending transactions on-chain, and share retrieval. Each operation was executed 100 times, and the results were averaged to determine the runtime. All experiments were conducted on a Microsoft Azure virtual machine configured as a **Standard DC4s v3 instance**, equipped with **4 vCPUs** and **32 GiB of RAM**.

### 4.1 Experimental Results

The following table summarizes the average runtime of each operation executed both in a standard environment (without TEE) and in a TEE-enabled environment. The overhead percentage shows the additional runtime cost due to TEE integration.

| Operation                  | Standard Environment Runtime Average (ms) | TEE-Enabled Runtime Average (ms) | Overhead (%) |
|----------------------------|------------------------------------|--------------------------|--------------|
| Key Extraction             | 1.5015                             | 1.5285                   | +1.80%       |
| Signing & Sending Keyshare on-Chain | 81.8016                            | 89.7137                  | +9.68%       |
| Share Retrieval            | 40.9942                            | 44.2842                  | +8.02%       |

### 4.2 Overhead Analysis

The observed results indicate that integrating TEE incurs some additional runtime overhead across all operations. Specifically, **key extraction** has a minimal increase in runtime of **1.80%**, showing that the overhead caused by TEE integration for this operation is negligible. For **signing and sending transactions on-chain**, the overhead was **9.68%**, which represents the highest observed increase among the tested operations. The **share retrieval** operation also experienced an overhead of **8.02%**.

These findings suggest that while using a TEE does introduce a runtime cost, this overhead is limited to a maximum of **9.68%** across the cryptographic operations. This level of overhead is acceptable given the enhanced security guarantees provided by TEE integration. Overall, the results demonstrate that TEE integration maintains an efficient framework suitable for critical operations, including key extraction, signing, and share decryption, even when interacting with blockchain networks.
