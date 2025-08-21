# HTTP Client (Rust)

A simple yet powerful command-line and TUI (Terminal User Interface)
based HTTP client written in Rust.
It allows you to send `GET`, `POST`, `PUT`, and `DELETE` requests with
custom headers and bodies, validate and pretty-print JSON, and explore
responses interactively in a terminal UI.

------------------------------------------------------------------------

## ✨ Features

-   📡 Supports **GET, POST, PUT, DELETE** requests
-   📝 Add custom headers in `Key: Value` format
-   📦 Send raw data or JSON payloads (with validation)
-   🎨 Pretty-prints JSON responses
-   🖥️ Interactive **TUI mode** for crafting and sending requests
-   📜 Request/Response history tracking in TUI
-   ⚡ Asynchronous, powered by **Tokio** + **Reqwest**

------------------------------------------------------------------------

## 📂 Project Structure

    rohan-choudharyy-http-client/
    ├── Cargo.toml         # Project configuration and dependencies
    └── src/
        ├── headers.rs     # Header parsing & validation
        ├── json.rs        # JSON validation & pretty printing
        ├── main.rs        # CLI entrypoint & logic
        └── tui.rs         # Interactive TUI implementation

------------------------------------------------------------------------

## 🚀 Getting Started

### 1. Clone the repo

``` bash
git clone https://github.com/your-username/http-client.git
cd http-client
```

### 2. Build

``` bash
cargo build --release
```

### 3. Run

``` bash
cargo run -- <command> [options]
```

------------------------------------------------------------------------

## 🛠 CLI Usage

``` bash
http <METHOD> <URL> [OPTIONS]
```

### Examples

-   **GET request**

``` bash
http get https://httpbin.org/get
```

-   **GET with headers**

``` bash
http get https://httpbin.org/get -H "Authorization: Bearer token"
```

-   **POST with raw data**

``` bash
http post https://httpbin.org/post --data "Hello World"
```

-   **POST with JSON**

``` bash
http post https://httpbin.org/post --json '{"name": "Alice"}'
```

-   **PUT with headers & JSON**

``` bash
http put https://httpbin.org/put -H "Content-Type: application/json" --json '{"id": 1, "status": "active"}'
```

-   **DELETE request**

``` bash
http delete https://httpbin.org/delete
```

------------------------------------------------------------------------

## 🎛️ TUI Mode

Launch the interactive terminal UI:

``` bash
http tui
```

### TUI Controls

-   `u` → Edit URL\
-   `h` → Edit Headers\
-   `b` → Edit Body\
-   `j` → Toggle JSON body mode\
-   `m` / `M` → Cycle HTTP method forward/backward\
-   `Enter` → Send request\
-   `Tab` → Switch between panels (Request / Response / History)\
-   `q` → Quit

------------------------------------------------------------------------

## 📦 Dependencies

-   [reqwest](https://crates.io/crates/reqwest) -- HTTP client
-   [tokio](https://crates.io/crates/tokio) -- Async runtime
-   [clap](https://crates.io/crates/clap) -- CLI argument parsing
-   [serde_json](https://crates.io/crates/serde_json) -- JSON parsing
-   [ratatui](https://crates.io/crates/ratatui) -- TUI framework
-   [crossterm](https://crates.io/crates/crossterm) -- Terminal handling
-   [tokio-util](https://crates.io/crates/tokio-util)

------------------------------------------------------------------------

## 📝 License

MIT License © 2025 Rohan Choudhary
