# Jetson_Line Simulator

A Rust implementation of the Jetson_Line 1D lattice for ethical neuromorphic simulation.
Models CHURCH, FEAR, POWER, TECH, load, trust, occupancy with bounded updates and invariants.
Integrates deed logging for moral judgement and pain metrics for "teachable discomfort".

## Features
- 1D lattice dynamics with per-site state updates and global invariants.
- PainMetrics for overload, trust drops, and repair necessity in [0,1].
- FearRegime transitions for adaptive FEAR bands.
- Deed logging for tamper-evident moral tracking.
- CLI demo for running episodes and querying metrics.

## Usage
cargo run -- --simulate-episodes 5 --lattice-size 50
cargo run -- --compute-pain "episode_1.json"

This contributes to TECH by simulating ethical dynamics, NANO by bounding states,
and POWER by enabling restorative analysis without harm.
