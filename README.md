# mini-Redis scaffold

This repository contains the file structure and learning notes for the project
described in `rust-mini-redis-prd.md`.

No Rust implementation has been added. The Rust files contain comments only:
each planned function has a stable reference code, its intended input/output,
its purpose, and the reason it belongs in that module.

## Files to complete

- `src/main.rs`: startup, configuration, logging, and runtime ownership.
- `src/resp/`: RESP2 frame model, incremental parser, and encoder.
- `src/store/`: entries, TTL behavior, LRU behavior, and shared state.
- `src/commands.rs`: command grammar and dispatch.
- `src/server.rs`: listener, per-client loop, pipelining, and sweeper.
- `tests/`: protocol integration and concurrent-client checks.
- `benches/`: optional parser benchmark.

## Dependency plan

Add dependencies gradually in `Cargo.toml`: Tokio for async networking,
bytes for buffers, clap for configuration, lru for V1 eviction, tracing for
logs, and tracing-subscriber for log output. Add dashmap only for the later V2
comparison; add criterion and tokio-test when you begin benchmarking and async
test work.

Ask about any reference code, such as `RESP-PARSER-003` or `STORE-006`, when
you are ready to implement that function.