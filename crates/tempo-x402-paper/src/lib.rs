//! Library surface for `tempo-x402-paper` — the benchmark harness and
//! candle-based inference backends. Also used by the `rust-agent` binary.
//!
//! Most consumers will only need `backends::candle::CandleGenerator` and the
//! `runner::CodeGenerator` trait.

pub mod backends;
pub mod humaneval;
pub mod results;
pub mod runner;
pub mod selfplay;
