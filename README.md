# 🦀 DynaRust Client (Official Rust SDK)

The official, asynchronous, and type-safe Rust client for [DynaRust](https://github.com/yourfavDev/DynaRust) — the distributed, horizontally scalable key-value store. 

This library provides a seamless wrapper around the DynaRust REST API, allowing you to interact with your cluster using strongly-typed Rust structs, automatic JSON deserialization, and async/await syntax 🔄.

---

## ✨ Key Features

* **🚀 Async by Default:** Built on top of `tokio` and `reqwest` for non-blocking, high-performance I/O.
* **📦 Strongly Typed:** Uses standard `serde` traits. Fetch data from DynaRust and deserialize it directly into your own custom Rust structs!
* **🔒 Built-in Authentication:** Easily manage and attach JWT tokens for secure `PUT` and `DELETE` operations.
* **🛡️ Error Handling:** Meaningful, standardized `DynaError` types (e.g., `NotFound`, `Unauthorized`) so you don't have to parse raw HTTP status codes.

---

## 📦 Installation

Add the client to your project using Cargo:

```bash
cargo add dynarust_client
```

## Basic usage
```rust
use dynarust_client::structs::{DynaClient, DynaError};
use serde::{Deserialize, Serialize};

// 1️⃣ Define your custom data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
struct UserProfile {
    username: String,
    level: u32,
    is_active: bool,
}

#[tokio::main]
async fn main() -> Result<(), DynaError> {
    // 2️⃣ Initialize the client pointing to any node in your cluster
    let mut client = DynaClient::new("http://localhost:6660");

    // 3️⃣ Authenticate (Registers if new, logs in if exists. Token is saved in client)
    client.auth("player_1", "super_secret_password").await?;
    println!("✅ Authenticated successfully!");

    // 4️⃣ Write Data (PUT)
    let profile = UserProfile {
        username: "player_1".to_string(),
        level: 42,
        is_active: true,
    };
    
    let saved_record = client.put_value("users", "profile_data", &profile).await?;
    println!("📝 Data saved! Version: {}", saved_record.version);

    // 5️⃣ Read Data (GET)
    // We map the incoming JSON directly back into our UserProfile struct
    let fetched = client.get_value::<UserProfile>("users", "profile_data").await?;
    println!("🔍 Fetched level: {}", fetched.value.level);

    // 6️⃣ Delete Data (DELETE)
    client.delete_value("users", "profile_data").await?;
    println!("🗑️ Data deleted!");

    Ok(())
}
```

## Real-Time Subscriptions (SSE)
```rust
use futures_util::StreamExt; // Required for stream iteration

#[tokio::main]
async fn main() {
    let client = DynaClient::new("http://localhost:6660");

    // Subscribe to a key. Returns an async Stream of your typed structs!
    let mut stream = client.subscribe::<UserProfile>("users", "profile_data").await.unwrap();

    println!("🎧 Listening for live updates...");

    // Process updates in real-time as they happen across the cluster
    while let Some(update) = stream.next().await {
        match update {
            Ok(record) => println!("🔥 LIVE UPDATE: {} leveled up to {}!", record.value.username, record.value.level),
            Err(e) => eprintln!("Stream error: {}", e),
        }
    }
}
```
📡 API Reference
DynaClient::new(base_url: &str)

Creates a new client instance. You can point this to any active node in your DynaRust cluster.

client.auth(user: &str, secret: &str)

Registers or logs in a user. If successful, automatically securely stores the returned JWT token inside the DynaClient instance for future PUT and DELETE requests.

client.set_token(token: String)

Manually attach a JWT authentication token to the client (useful if you handle authentication elsewhere).

client.get_value::<T>(table: &str, key: &str)

Retrieves the latest version of a key from the specified table. Does not require authentication.

Returns: Result<VersionedValue<T>, DynaError>

client.put_value<T>(table: &str, key: &str, value: &T)

Creates or updates a key. Requires the client to be authenticated as the record's owner.

Returns: Result<VersionedValue<T>, DynaError>

client.delete_value(table: &str, key: &str)

Deletes a key from the table. Requires the client to be authenticated as the record's owner.

Returns: Result<(), DynaError>

client.subscribe::<T>(table: &str, key: &str)

Opens a Server-Sent Events (SSE) connection to the cluster.

Returns: An asynchronous Stream that yields Result<VersionedValue<T>, DynaError> whenever the key is modified.

🆘 Troubleshooting
Error: Request failed: error sending request... connection refused
Make sure your DynaRust node is running and accessible at the URL provided to DynaClient::new(). If running in Docker, ensure your ports are mapped correctly.

Error: Unexpected status 401: Unauthorized access
You attempted a write/delete operation without authenticating. Make sure to call client.auth() or client.set_token() before making the request.

Error: trait bound is not satisfied
Ensure the struct you are passing to generic methods has #[derive(Serialize, Deserialize)] attached to it.

🤝 Contributing

Pull requests are welcome! If you find a bug or want to help expand this client, feel free to open an issue or submit a PR on the main repository.


***

### Next Step
To make the SSE (`subscribe`) and other missing functions actually work in your Rust code, we'll need to update `src/structs.rs` and add `reqwest-eventsource` and `futures-util` to your `Cargo.toml`. 

Would you like me to write out the updated `src/structs.rs` file containing the `auth`, `put_value`, `delete_value`, and `subscribe` implementations?