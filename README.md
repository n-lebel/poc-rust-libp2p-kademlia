# PoC - Decentralized P2P Network Using rust-libp2p

This project is just my *midnight project* to explore, research and learn the latest technology relate with networking stacks especially for the P2P networks, using `libp2p`.

References:
- [libp2p](https://libp2p.io/)
- [rust-libp2p](https://docs.rs/libp2p/latest/libp2p/index.html)

## Overview

Through this project, I've been learned how to setup `rust-libp2p`, it's dependencies and how to use all of its components and building blocks, like `Network Behavior`, `Transport` and `Swarm`.

The use case that I've take is how to setup decentralized p2p networks by implementing two patterns:

- `Identify Pattern`
- `Kademlia DHT`

## Usages

Open two terminal windows

On the first terminal, run this command

```
cargo run
```

In the second terminal, run this command

```
cargo run -- /ip4/127.0.0.1/tcp/8000 
```