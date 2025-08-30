# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Common Commands

*   **Build:** `cargo build`
*   **Run:** `cargo run`
*   **Test:** `cargo test`
*   **Lint:** `cargo clippy`
*   **Check:** `cargo check` (also available as `make check`)
*   **Run a single test:** `cargo test --test <TEST_NAME>`

## Code Architecture

This is a Rust-based web service built with the Axum web framework. It serves as a real-time component for the Sparrow application.

### Key Components:

*   **Web Framework:** Axum is used for handling HTTP requests.
*   **Asynchronous Runtime:** Tokio is the runtime for asynchronous operations.
*   **Database/Cache:** Redis is used for caching and potentially for real-time messaging.
*   **External Services:** The application interacts with Firebase services, including Firebase Cloud Messaging (FCM).
*   **Serialization/Deserialization:** Serde is used for handling JSON data.
*   **HTTP Client:** Reqwest is used for making outbound HTTP requests.

### Project Structure:

The main application logic is located in the `src` directory. The `main.rs` file is the entry point of the application. The project follows standard Rust conventions.

