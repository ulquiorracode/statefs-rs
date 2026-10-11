# statefs-adapter-bridge-config

Modern, high-performance configuration management engine for StateFS, featuring Ports & Adapters architecture, Figment-inspired merge policies, and C#-inspired trait capability composition.

## Features

- **Ports & Adapters Architecture**:
  - `ConfigStoragePort` & `ConfigStorageMut`: clean boundary isolating storage engines from configuration models.
  - `StateFsStorageAdapter`: zero-allocation, lockless in-memory storage adapter built directly atop StateFS `MemStore`.
- **Pluggable Multi-Source Engine (`ConfigSourcePort`)**:
  - Built-in adapters for `Defaults`, `DictSource` (key-value CLI/maps), `Toml`, `Json`, and `Env`.
  - Prefix mounting via `.at("custom/prefix")` combinator.
- **Explicit Push Policies**:
  - `PushPolicy::Merge` (`.merge(...)`): incoming source overrides existing keys.
  - `PushPolicy::Join` (`.join(...)`): existing configuration is preserved; source acts as default fallbacks.
- **Trait Capability Composition**:
  - `ConfigAccess`: unified generic `.get::<T>(path)` supporting zero-copy `&str`, primitive integers/floats, booleans, and Serde-deserializable structs.
  - `ConfigMut`: safe runtime mutation and deletion (`.set()`, `.remove()`).
  - `ConfigPersist`: save mutations back to storage.
- **Orthogonal SMA Validation Subsystem**:
  - `ConfigRule` SPI port for decoupled validation rules.
  - Declarative rules: `Rule::required`, `Rule::range`, `Rule::choices`, `Rule::custom`.
  - Compile-time pre-flight verification barrier during `.build()`.
  - Runtime mutation contract enforcement via `ValidatedConfig` (`.build_validated()`).
- **Scoped Sub-Builders**:
  - Modular nested configurations via `.scope("prefix", |sub| ...)`.
- **Zero-Copy Performance**:
  - Reads directly from contiguous StateFS tree memory without string heap allocations on query paths.

## Quick Start

```rust
use statefs_adapter_bridge_config::{
    ConfigAccess, ConfigManager, ConfigMut, Defaults, DictSource, Toml, Json, Env, Rule,
};

// 1. Build configuration pipeline with contract validation
let mut cfg = ConfigManager::builder()
    // Join defaults (fallback)
    .join(Defaults::new()
        .set("server/port", 27015)
        .set("server/hostname", "Default Server")
        .set("server/tickrate", 64))
    // Merge environment variables (GAME_* prefix)
    .merge(Env::prefixed("GAME_").separator("__"))
    // Attach orthogonal validation rules
    .rule(Rule::required("server/hostname"))
    .rule(Rule::range("server/port", 1.0, 65535.0))
    // Nested modular scope for plugins
    .scope("plugins", |sub| {
        sub.merge(Defaults::new().set("vip/enabled", true))
    })
    .build()?;

// 2. Generic zero-copy reading
let port: u16 = cfg.get("server/port")?;
let hostname: &str = cfg.get("server/hostname")?;
assert_eq!(port, 27015);

// 3. Runtime mutation (via ConfigMut capability)
cfg.set("server/tickrate", 128)?;
assert_eq!(cfg.get::<i64>("server/tickrate")?, 128);
```
