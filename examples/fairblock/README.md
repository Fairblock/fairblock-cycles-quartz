# Fairblock-Cycles-Quartz

Fairblock-Cycles-Quartz leverages the Cycles-Quartz libraries to implement a framework for Fairyring validators to run the FairyringClient functionality inside a TEE and get attested on-chain.

A CosmWasm contract is used for registration of the TEEs on Fairyring. The registration process envolves performing a handshake between the contract and an enclve. After a successful handshake, the contract stores the public key of the enclave in a list represnting the public keys of the registered TEEs. This list can later be queried for verification of the messages coming from inside the enclaves.

Once the validators are registered, their PKs are retrieved from the contract. Each validator's share is encrypted using their registered PK, and the encrypted shares are securely sent on-chain. The TEEs fetch these encrypted shares and perform decryption within the enclave using the corresponding secret key, ensuring that the validators themselves remain unaware of their shares.

The enclave actively monitors the chain for decryption key requests. Once a decryption key is requested, the enclave code first validates it against the chain state. To validate the chain state, the Tendermint's abci_query mechanism is used. Upon validation, the required key share is securely extracted within the enclave, signed using the enclave's SK (corresponding to the PK stored on-chain), and then submitted back to the Fairyring. This signature is verified on chain using the stored PKs in the contract to ensure that the message originates from within the TEE. Following this verification, the extracted key is used for key aggregation.
Below is a diagram of the steps:
![Fairblock-Cycles-Quartz](./cycles.png)
We used the `Transfers` example as the base for our implementation. We also modified the cli code to deploy the contract on Fairyring.

## Testing
There is a test script (`test.sh`) for performing an end-to-end testing of the process. For the SGX version, there is a `test-sgx.sh` which deploys the TCB and DCAP contracts and sets them up on Fairyring. The rest of the test script is fairly similar to the mock-SGX version.

## Benchmarks
| Case                        | Mock-SGX Average (ms) | SGX Average (ms) | Overhead (%)            |
|-----------------------------|-----------------------|-------------------|------------------------|
| Key Extraction              | 1.5015                | 1.5285            | +1.80%                 |
| Signing & Sending on Chain  | 81.8016               | 89.7137           | +9.68%                 |
| Get Share                   | 40.9942               | 44.2842           | +8.02%                 |

### Analysis

The table above compares the average runtimes for different operations executed in mock-SGX and SGX environments. The overhead percentage is calculated to show the additional cost caused by SGX in terms of runtime.

From the results:
- **Key Extraction**: SGX introduces an overhead of approximately **1.80%**, showing a small increase in runtime for extracting the keyshare for a requested identity.
- **Signing & Sending on Chain**: The overhead here is **9.68%** for the process of signing the extracted key and sending the tx on chain.
- **Get Share**: The SGX version has an **8.02%** overhead compared to mock-SGX in the process of fetching the share and decrypting it. 

These results demonstrate that using SGX causes a maximum of 9.68% runtime cost, particularly in complex operations like signing, decrypting shares and communicating with the chain.
