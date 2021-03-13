//! A middleware service for delivering messages in a causal order.
extern crate bincode;
extern crate bit_vec;
extern crate crossbeam;
extern crate serde;
#[macro_use]
extern crate serde_derive;
/**
 * Required broadcast API.
 */
pub mod broadcast;
/**
 * Causal verification from a broadcast results.
 */
pub mod causality_checker;
/**
 * Middleware configuration.
 */
pub mod configuration;
/**
 * Causal delivery middleware that uses a graph approach.
 */
pub mod graph;
/**
 * Causal delivery middleware that uses version vectors.
 */
pub mod vv;
