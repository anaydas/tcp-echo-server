use indicatif::{ProgressBar, ProgressStyle};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

struct Config {
    host: String,
    port: u16,
}

// Sets port number at which server listens and the host (localhost).
fn setupflag() -> Config {
    Config {
        host: String::from("127.0.0.1"),
        port: 7379,
    }
}

// Reads raw bytes from the stream. Returns error if client disconnects or read fails.
#[allow(non_snake_case)]
fn readCommand(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
    let mut buffer = [0u8; 1024];
    let bytes_read = stream.read(&mut buffer)?;

    if bytes_read == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionAborted,
            "Client disconnected",
        ));
    }

    // ---- Logging of raw bytes ----
    let hex: Vec<String> = buffer[..bytes_read]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    println!("🔸 Received {} bytes: {}", bytes_read, hex.join(" "));

    if let Ok(txt) = std::str::from_utf8(&buffer[..bytes_read]) {
        let txt = txt.replace("\r", "\\r").replace("\n", "\\n");
        println!("🔹 Interpreted as UTF‑8: {}", txt);
    }
    // ----------------------------

    Ok(buffer[..bytes_read].to_vec())
}

// Very tiny RESP parser – extracts command name and arguments
fn parse_resp_command(buf: &[u8]) -> Option<(String, Vec<String>)> {
    // Expect an array starting with '*'
    if buf.is_empty() || buf[0] != b'*' {
        return None;
    }
    // Split on CRLF (\r\n) – treat both \r and \n as separators for simplicity
    let tokens: Vec<&[u8]> = buf.split(|b| *b == b'\n').collect();
    let mut args: Vec<String> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i].starts_with(b"$") && i + 1 < tokens.len() {
            // strip possible trailing \r from the payload line
            let mut data = tokens[i + 1];
            if data.ends_with(b"\r") {
                data = &data[..data.len() - 1];
            }
            if let Ok(s) = std::str::from_utf8(data) {
                args.push(s.to_string());
            }
            i += 2;
        } else {
            i += 1;
        }
    }
    if args.is_empty() {
        None
    } else {
        let cmd = args[0].clone();
        Some((cmd, args[1..].to_vec()))
    }
}

// Displays an animated braille spinner for ~1.2s then prints the server-ready message.
fn show_startup_spinner(address: &str) {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} ☁️  Starting TCP echo server...")
            .unwrap()
            // tick_chars() mis-counts multi-byte Unicode — use tick_strings() instead
            .tick_strings(&["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈", "⠁"]),
    );
    pb.enable_steady_tick(Duration::from_millis(80));

    // bind() completes in microseconds; sleep so animation has time to play
    std::thread::sleep(Duration::from_millis(1200));

    pb.finish_and_clear();
    println!("✅  Server started! Listening on {}", address);
    println!();
}

#[allow(non_snake_case)]
fn RunSyncTCPServer(config: Config) {
    let address = format!("{}:{}", config.host, config.port);

    show_startup_spinner(&address);

    let listener = TcpListener::bind(&address).expect("Could not bind to address");

    let mut concurrent_connection: u32 = 0;

    // Listens and accepts request continuously
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                concurrent_connection += 1;
                println!("New connection established. Concurrent connections: {}", concurrent_connection);

                if let Ok(peer_addr) = stream.peer_addr() {
                    println!("Accepted connection from: {}", peer_addr);
                }

                loop {
                    match readCommand(&mut stream) {
                        Ok(payload) => {
                            // Try to interpret as RESP command
                            if let Some((cmd, _args)) = parse_resp_command(&payload) {
                                if cmd.eq_ignore_ascii_case("COMMAND") {
                                    // Reply with an empty array to COMMAND and COMMAND DOCS
                                    // This prevents redis-cli from trying to parse strings as maps/arrays
                                    let resp = b"*0\r\n";
                                    if let Err(e) = stream.write_all(resp) {
                                        eprintln!("Error writing RESP reply: {}", e);
                                        break;
                                    }
                                    continue; // wait for next command
                                }
                            }

                            // Format the raw payload into a human-readable single line, replacing \r\n with spaces
                            // For example, "*3\r\n$3\r\nSET\r\n$1\r\nK\r\n$1\r\nV\r\n" becomes "*3 $3 SET $1 K $1 V"
                            let readable_payload = String::from_utf8_lossy(&payload).replace("\r\n", " ");
                            let trimmed = readable_payload.trim();
                            
                            // Send it back as a RESP Simple String so redis-cli prints it perfectly
                            let resp = format!("+{}\r\n", trimmed);

                            if let Err(e) = stream.write_all(resp.as_bytes()) {
                                eprintln!("Error writing to stream: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Connection closed or error: {}", e);
                            break; // Break the loop to return and handle the next connection
                        }
                    }
                }

                concurrent_connection -= 1;
                println!("Connection closed. Concurrent connections: {}. Waiting for next...\n", concurrent_connection);
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

fn main() {
    let config = setupflag();
    RunSyncTCPServer(config);
}
