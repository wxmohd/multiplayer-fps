use std::net::UdpSocket;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multiplayer FPS Server ===");
    
    let socket = UdpSocket::bind("127.0.0.1:34254")?;
    println!("Server listening on 127.0.0.1:34254");
    println!("Waiting for clients to connect...");
    
    let mut buf = [0; 1024];
    
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, src)) => {
                let msg = String::from_utf8_lossy(&buf[..size]);
                println!("Received from {}: {}", src, msg);
                
                // Echo back for now
                let response = format!("Server received: {}", msg);
                socket.send_to(response.as_bytes(), src)?;
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
            }
        }
    }
}