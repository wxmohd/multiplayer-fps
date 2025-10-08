use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    println!("=== Multiplayer FPS Server ===");

    let socket = UdpSocket::bind("0.0.0.0:34254")?;
    socket.set_nonblocking(true)?;
    println!("Server listening on {}", socket.local_addr()?);

    let mut buf = [0; 2048];
    let mut clients: HashMap<String, std::net::SocketAddr> = HashMap::new();
    let mut last_seen: HashMap<String, Instant> = HashMap::new();

    loop {
        if let Ok((n, src)) = socket.recv_from(&mut buf) {
            if n == 0 {
                continue;
            }
            let msg = String::from_utf8_lossy(&buf[..n]).to_string();
            if msg.starts_with("CONNECT:") {
                let name = msg[8..].to_string();
                println!("{} joined from {}", name, src);
                clients.insert(name.clone(), src);
                last_seen.insert(name.clone(), Instant::now());

                // broadcast JOIN
                for addr in clients.values() {
                    let _ = socket.send_to(format!("JOIN:{}", name).as_bytes(), addr);
                }
            } else if msg == "DISCONNECT" {
                if let Some((name, _)) = clients.iter().find(|(_, v)| **v == src).map(|(k, v)| (k.clone(), *v)) {
                    println!("{} left", name);
                    clients.remove(&name);
                    last_seen.remove(&name);

                    for addr in clients.values() {
                        let _ = socket.send_to(format!("LEAVE:{}", name).as_bytes(), addr);
                    }
                }
            } else if msg.starts_with("STATE:") {
                if let Some((name, _)) = clients.iter().find(|(_, v)| **v == src).map(|(k, v)| (k.clone(), *v)) {
                    last_seen.insert(name.clone(), Instant::now());
                    // forward to everyone except sender
                    for (peer_name, addr) in &clients {
                        if peer_name != &name {
                            let _ = socket.send_to(format!("STATE:{}:{}", name, &msg[6..]).as_bytes(), addr);
                        }
                    }
                }
            } else if msg == "PING" {
                let _ = socket.send_to(b"PONG", src);
            }
        }

        // prune inactive clients
        let now = Instant::now();
        clients.retain(|name, _| {
            if let Some(last) = last_seen.get(name) {
                now.duration_since(*last).as_secs() < 20
            } else {
                false
            }
        });
    }
}
