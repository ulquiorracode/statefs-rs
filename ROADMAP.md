# StateFS Roadmap

All major milestones, architectural goals, and progress tracking for the **StateFS** ecosystem.

---

## Milestone v0.1.0 — Foundational Virtual State Engine [Completed]

**Goal:** Establish the nanokernel `#![no_std]` core, virtual inode addressing, and baseline memory providers.

- [x] `#![no_std]` core engine (`statefs-core`).
- [x] Virtual hierarchical path resolution ($O(K)$ segment traversal).
- [x] Zero-allocation memory provider with contiguous node storage.
- [x] Basic synchronization locks with reader-writer poison recovery.
- [x] Comprehensive unit and integration test suite.

---

## Milestone v0.2.0 — Port-Adapter-Provider Triad & Config Separation (Current)

**Goal:** Cleanly decouple ecosystem adapters, configuration subsystems, and storage providers according to canonical SMA principles.

- [x] Separate `statefs-config` into an autonomous, typed configuration management crate.
- [x] Rename legacy hubs to canonical `statefs-adapter-bridge-config` and `statefs-adapter-axum`.
- [x] Add Axum HTTP virtual filesystem inspector for live state debugging.
- [x] Implement append-only zero-alloc Write-Ahead Log (WAL) drain buffer.
- [x] Multi-platform CI pipeline (Ubuntu, Windows, macOS) with Clippy and audit checks.
- [ ] Stabilize dynamic provider hot-plugging SPI.

---

## Milestone v0.3.0 — Distributed Sync & Kernel Runtimes (Planned)

**Goal:** Extend StateFS to WASM components, eBPF telemetry hooks, and distributed raft state replication.

- [ ] WebAssembly Component Model bindings for zero-copy sandbox state exchange.
- [ ] eBPF virtual probe provider for Linux kernel runtime state observation.
- [ ] Reactive pub-sub node watchers via non-allocating event channels.
- [ ] Formal mathematical verification of lock-free RCU swap operations.
