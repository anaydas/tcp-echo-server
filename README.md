# TCP Echo Server (Rust)

A **simple, single‑threaded TCP echo server** written in Rust, with a nice animated spinner using the `indicatif` crate when the server starts up.  The server accepts TCP connections, echoes back any data received, and tracks the number of concurrent connections (always `0` or `1` in this single‑threaded implementation).

---

## ✨ Features
- **Echo functionality** – whatever the client sends is sent back unchanged.
- **Connection counter** – prints `New connection established...` and `Connection closed...` for each client.
- **Zero‑dependency client** – you can test it with the built‑in `nc` (netcat) command.

---

## 📦 Prerequisites
- Rust toolchain (stable ≥ 1.70) – install via [`rustup`](https://rustup.rs/).
- Cargo (automatically bundled with Rust).
- (Optional) `nc` / `netcat` for quick testing.

---

## 🛠️ Build & Run
```bash
# Clone / navigate to the repository (already in your workspace)
cd /Users/user/Documents/workspace/tcp-echo-server

# Build the binary (debug build is fine for learning)
cargo build

# Run the server – it will show the spinner then report the listening address
cargo run
```
The server will start on **`127.0.0.1:7379`** (default port defined in `setupflag`).  You can change the port by editing the `setupflag()` function in `src/main.rs`.

---

## 📡 Test with Netcat
Open a second terminal and run:
```bash
nc 127.0.0.1 7379
```
Type any text and press **Enter** – the same text will be echoed back:
```
hello
hello
```
Press **Ctrl‑C** to close the client; the server will log the connection closure.

---

## 📁 Project Structure
```
├─ Cargo.toml          # Cargo manifest (includes indicatif dependency)
├─ README.md           # ← this file
└─ src
   ├─ main.rs          # Application entry point, spinner, server loop
   └─ (other modules)  # currently all code lives in main.rs for simplicity
```

---

## 🐛 Troubleshooting
- **Port already in use** – kill the existing process (`lsof -ti :7379 | xargs kill -9`) or change the port number.
- **Spinner not visible** – ensure you have a recent version of `indicatif` (≥ 0.17) and that the terminal supports UTF‑8.

---

## 📜 License
This example is provided under the **MIT License** – feel free to copy, modify, and experiment.

---

Enjoy learning Rust networking! 🚀
