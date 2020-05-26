[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](
https://github.com/carlospereira1607/TCB)
[![Rust 1.36+](https://img.shields.io/badge/rust-1.36+-lightgray.svg)](
https://www.rust-lang.org)
[![Cargo](https://img.shields.io/badge/crates.io-0.1.20-orange)](
https://crates.io/crates/tcb)
[![Cargo](https://img.shields.io/badge/docs-0.1.20-blue)](
https://docs.rs/tcb/0.1.20/tcb/)

# TCB
A Tagged Causal Broadcast middleware service in Rust.

Offers both a graph based and a version vector approach middleware that delivers messages in a causal order. This middleware only works for full-mesh topologies, where the peers already know each other's addresses and the ports they're listening to before connecting. 

This service ensures message delivery in a causal order and also allows to track causal stability of messages.

For now, only a thread based implementation exists. For every peer in the group, the middleware creates a pair of Reader/Sender threads. The reason for this pair is so that the middleware doesn't have to know the order by which it should establish connections.

Besides the middleware service, a causality checker was created that verifies, at the end of the broadcast, if the messages were correctly marked as delivered and stable at every peer. To do this, every peer's dot sequence must be passed to the checker, which is a backtracking algorithm that supports checking causality for both the GRAPH and VV approaches.

## Causal Delivery
There are many consistency models, each one defining how a system should behave in certain situations. This crate focuses on causal consistency. This model aims to capture causal relationships between events in the system, where the processes only observe causally related events in the same causal order. That is, every node in the system agrees on the causal order of related events. 

There are stronger models like sequential consistency, but for this model it is required that all the events appear in the same total order for every process, but a trade-off is implicit with this: the system’s availability is sacrificed for a stronger consistency assurance. Therefore, even though it is weaker, causal consistency is the best option for achieving high-availability and high-responsiveness when network partitions and failures occur.

## Causal Stability

Suppose a message *m* tagged with a causal timestamp *t* that was delivered at a process *p1*. This message *m* will be considered causally stable when all subsequently delivered messages at *p1* have a causal timestamp *t’* such that:

```
t' > t
```

This means that *m* is considered to be causally stable when no messages concurrent to *m* will be delivered to *p1* after *m*.

The concept of causal stability can be used for garbage collection or message retransmission. However, this is a middleware configuration parameter that can be turned off, since calculating stability adds some delay on message delivery.

## Usage

First add this to your `Cargo.toml`:

```toml
[dependencies]
tcb = "0.1.201"
```

Before a middleware instance can be created, each peer must have the following:

- A globally unique id
- A list of addresses and ports to connect to
- A port to listen for connections
- The middleware configuration file 

The middleware configuration file is in the TOML format and the peers in the group must have a unique id that's represented as integer, starting at 0 and incrementing with each peer. Furthermore, messages must be serialized before sending over the TCP network.

The `TCB` trait was added to simplify creating generic code that uses the middleware, regardless of implementation. Therefore, it must be imported, alongside the `middleware_configuration` and the `GRAPH`/`VV` modules.  



## Examples

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