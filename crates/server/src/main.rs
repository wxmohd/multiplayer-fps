use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::Instant;

#[derive(Clone)]
struct PlayerInfo {
    addr: std::net::SocketAddr,
    spawn_x: f32,
    spawn_y: f32,
}

// Predefined spawn positions for different players
const SPAWN_POSITIONS: [(f32, f32); 8] = [
    (100.0, 100.0),   // Player 1 - top-left area
    (500.0, 500.0),   // Player 2 - bottom-right area  
    (100.0, 500.0),   // Player 3 - bottom-left area
    (500.0, 100.0),   // Player 4 - top-right area
    (300.0, 100.0),   // Player 5 - top-center
    (300.0, 500.0),   // Player 6 - bottom-center
    (100.0, 300.0),   // Player 7 - left-center
    (500.0, 300.0),   // Player 8 - right-center
];

fn main() -> std::io::Result<()> {
    println!("=== Multiplayer FPS Server ===");

    let socket = UdpSocket::bind("0.0.0.0:34254")?;
    socket.set_nonblocking(true)?;
    println!("Server listening on {}", socket.local_addr()?);

    let mut buf = [0; 2048];
    let mut clients: HashMap<String, PlayerInfo> = HashMap::new();
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
                
                // Assign spawn position based on player count
                let player_index = clients.len() % SPAWN_POSITIONS.len();
                let (spawn_x, spawn_y) = SPAWN_POSITIONS[player_index];
                
                let player_info = PlayerInfo {
                    addr: src,
                    spawn_x,
                    spawn_y,
                };
                
                clients.insert(name.clone(), player_info);
                last_seen.insert(name.clone(), Instant::now());

                // Send spawn position to the new player
                let spawn_msg = format!("SPAWN:{}:{}", spawn_x, spawn_y);
                let _ = socket.send_to(spawn_msg.as_bytes(), src);

                // broadcast JOIN to all players
                for player_info in clients.values() {
                    let _ = socket.send_to(format!("JOIN:{}", name).as_bytes(), &player_info.addr);
                }
            } else if msg == "DISCONNECT" {
                if let Some((name, _)) = clients.iter().find(|(_, v)| v.addr == src).map(|(k, v)| (k.clone(), v.clone())) {
                    println!("{} left", name);
                    clients.remove(&name);
                    last_seen.remove(&name);

                    for player_info in clients.values() {
                        let _ = socket.send_to(format!("LEAVE:{}", name).as_bytes(), &player_info.addr);
                    }
                }
            } else if msg.starts_with("STATE:") {
                if let Some((name, _)) = clients.iter().find(|(_, v)| v.addr == src).map(|(k, v)| (k.clone(), v.clone())) {
                    last_seen.insert(name.clone(), Instant::now());
                    println!("📡 Received STATE from {}: {}", name, &msg[6..]);
                    println!("👥 Connected clients: {}", clients.len());
                    
                    // forward to everyone except sender
                    let mut forwarded_count = 0;
                    for (peer_name, player_info) in &clients {
                        if peer_name != &name {
                            let forward_msg = format!("STATE:{}:{}", name, &msg[6..]);
                            println!("📤 Forwarding to {}: {}", peer_name, forward_msg);
                            let _ = socket.send_to(forward_msg.as_bytes(), &player_info.addr);
                            forwarded_count += 1;
                        }
                    }
                    println!("✅ Forwarded to {} clients", forwarded_count);
                } else {
                    println!("❌ Unknown client sent STATE: {}", src);
                }
            } else if msg.starts_with("SHOOT:") {
                if let Some((name, _)) = clients.iter().find(|(_, v)| v.addr == src).map(|(k, v)| (k.clone(), v.clone())) {
                    last_seen.insert(name.clone(), Instant::now());
                    // forward to everyone except sender
                    for (peer_name, player_info) in &clients {
                        if peer_name != &name {
                            let _ = socket.send_to(format!("SHOOT:{}:{}", name, &msg[6..]).as_bytes(), &player_info.addr);
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
