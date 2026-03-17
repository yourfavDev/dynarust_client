# GEMINI Context: DynaRust Client (Rust SDK)

This file provides critical context for Gemini CLI when working within the `dynarust_client` repository.

## 🚀 Project Overview

**DynaRust Client** is the official, asynchronous, and type-safe Rust SDK for [DynaRust](https://github.com/yourfavDev/DynaRust), a distributed, horizontally scalable key-value store. 

- **Language:** Rust (Edition 2021)
- **Core Stack:** `tokio` (Async runtime), `reqwest` (HTTP client), `serde` (Serialization), `reqwest-eventsource` (SSE for real-time updates).
- **Architecture:** The library consists of a main `DynaClient` that handles authentication (JWT), standard REST operations (GET, PUT, DELETE), and real-time subscriptions using Server-Sent Events (SSE).

## 📁 Key Files

- `Cargo.toml`: Project metadata and dependencies.
- `src/lib.rs`: Entry point, exports the `models` module.
- `src/models.rs`: Contains the core `DynaClient` implementation, data models (`VersionedValue`), and error handling (`DynaError`).
- `README.md`: Extensive documentation and usage examples.

## 🛠 Building and Running

### Commands
- **Build:** `cargo build`
- **Test:** `cargo test`
- **Documentation:** `cargo doc --open`
- **Linting:** `cargo clippy`
- **Formatting:** `cargo fmt`

### Typical Workflow
The client is intended to be used as a dependency in other Rust projects. To test changes locally, you can use `cargo test` (ensure a local DynaRust node is running if integration tests are added).

## 📝 Development Conventions

- **Async/Await:** All I/O operations must be asynchronous using `tokio`.
- **Type Safety:** Use generics (`<T>`) for data operations to allow users to deserialize directly into their own structs.
- **Error Handling:** Centralized in `DynaError`. Prefer returning `Result<T, DynaError>` for all client methods.
- **Serde:** All models and user-provided data must implement `Serialize` and `Deserialize`.
- **JWT Management:** The `DynaClient` automatically manages the JWT token once `auth()` is called. Always check for the presence of a token before performing restricted operations (PUT/DELETE).
- **SSE Streams:** The `subscribe` method returns an asynchronous `Stream`. Use `futures-util` or `tokio-stream` to consume it.

## ⚠️ Known Limitations / TODOs
- **URL Encoding:** Currently, table and key names are formatted directly into URLs. For production, these should be URL-encoded to handle special characters.
- **Retries:** Basic client implementation lacks automatic retries for failed requests.
- **Mocking:** There is currently no built-in mocking for testing without a live DynaRust cluster.
