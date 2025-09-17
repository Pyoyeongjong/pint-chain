# pint-chain
Mini BlockChakin Layer 1 Project

## Project Overview
- Layer 1 Blockchain implemented in Rust
- Account-based blockchain
- Five main components: (`Network`, `Consensus (BlockImporter + Miner)`, `PayloadBuilder`, `Pool (TxPool + Validator)`, `Provider (StateProvider + DB)`) + `RPC Server`
- Consensus Rule: Proof of Work (PoW)
- Difficulty adjustment: doubled or halved every block
- Fork Choice: Longest Chain Rule
- P2P Discovery: initially boot node, if full then random peer from boot node
- World module that describes the game field (not yet implemented)
- RPC: HTTP Server–Client communication
- Heavily inspired by [reth](https://github.com/paradigmxyz/reth)
- ECDSA + Signer Recovery (same as ethereum)

## How to use
### Run a node
```bash
cargo run -- --boot-node
cargo run -- --name A --port 30304 --rpc-port 8546 --miner-address 0534501c34f5a0f3fa43dc5d78e619be7edfa21a
```

### Rpc connectioin
Use [clients/rpc](clients/rpc) crate. 
Currently only responds with binary data (example included).

### Execution examples
**Boot_node: node start**
<img width="1919" height="1027" alt="image" src="https://github.com/user-attachments/assets/90094f23-d22a-4af3-a323-a012c700e4ae" />
**Boot_node: connection with normal node a**
<img width="1112" height="379" alt="image" src="https://github.com/user-attachments/assets/0321aef8-514b-46a2-ac22-b55849292300" />

**Normal node a: node start + p2p connection with boot_node + syncronizing with boot node**
<img width="1919" height="1018" alt="image" src="https://github.com/user-attachments/assets/193ed51b-4327-4582-9ee9-76b07532c0f0" />

## TODO LIST
1. Implement World (Whatever! maybe game?)
2. Block Explorer
