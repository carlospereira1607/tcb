[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](
https://github.com/carlospereira1607/TCB)
[![Rust 1.28+](https://img.shields.io/badge/rust-1.28+-lightgray.svg)](
https://www.rust-lang.org)
[![Cargo](https://img.shields.io/badge/crates.io-0.1.20-orange)](
https://crates.io/crates/tcb)
[![Cargo](https://img.shields.io/badge/docs-0.1.20-blue)](
https://docs.rs/tcb/0.1.20/tcb/)

# TCB
A Tagged Causal Broadcast middleware service in Rust.

Offers both a graph based and a version vector approach middleware that delivers messages in a causal order. This middleware only works for full-mesh topologies, where the peers already know each other's addresses and the ports they're listening to before connecting. 

Before creating a middleware instance, each peer must have the following:

- A globally unique id
- A list of addresses and ports to connect to
- A port to listen for connections
- The middleware configuration file 

The middleware configuration file is in the TOML format and the peers in the group must have a unique id that's represented as integer, starting at 0 and incrementing with each peer. Furthermore, messages must be serialized before sending over the TCP network.

The `TCB` trait was added to simplify creating generic code that uses the middleware, regardless of implementation. Therefore, it must be imported, alongside the `middleware_configuration` and the `GRAPH`/`VV` modules.  

For now, only a thread based implementation exists. For every peer in the group, the middleware creates a pair of Reader/Sender threads. The reason for this pair is so that the middleware doesn't have to know the order by which it should establish connections.

Besides the middleware service, a causality checker was created that verifies, at the end of the broadcast, if the messages were correctly marked as delivered and stable at every peer. To do this, every peer's dot sequence must be passed to the checker, which is a backtracking algorithm that supports checking causality for both the GRAPH and VV approaches.

Examples of peers, configuration file and causality checker can be found [here](https://github.com/carlospereira1607/TCB/tree/master/examples).


## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.