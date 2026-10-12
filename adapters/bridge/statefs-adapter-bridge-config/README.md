# statefs-adapter-bridge-config

High-performance StateFS compatibility bridge for the [`config`](https://crates.io/crates/config) (`config-rs`) ecosystem.

## Features

- **`config::Source` Implementation**: Export any StateFS `MemStore` virtual tree directly into `config::ConfigBuilder`.
- **`ConfigBridge`**: Lightweight wrapper reading StateFS state using `config-rs` ergonomics without string heap allocations on query paths.

## Quick Start

```rust
use statefs_adapter_bridge_config::{ConfigBridge, StateFsSource};
use statefs_core::{MemStore, Path, Store, Value};

let mut store = MemStore::new();
store.insert(&Path::parse("/server/port"), Value::from(27015))?;

// Use StateFS as a source in config-rs
let config = config::Config::builder()
    .add_source(StateFsSource::new(store))
    .build()?;

assert_eq!(config.get::<i64>("server.port")?, 27015);
```
