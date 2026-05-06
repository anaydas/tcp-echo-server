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

// Reads command from the stream. Returns error if client disconnects or read fails.
#[allow(non_snake_case)]
fn readCommand(stream: &mut TcpStream) -> Result<String, std::io::Error> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    
    if bytes_read == 0 {
        // Connection closed by client
        return Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionAborted,
            "Client disconnected",
        ));
    }
    
    Ok(String::from_utf8_lossy(&buffer[..bytes_read]).into_owned())
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
                        Ok(message) => {
                            print!("Received message: {}", message);

                            // Respond with the same message
                            if let Err(e) = stream.write_all(message.as_bytes()) {
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
