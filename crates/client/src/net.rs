use std::net::UdpSocket;
use std::time::{Duration, Instant};

pub struct NetClient {
    pub socket: UdpSocket,
    pub server: String,
    pub username: String,
    last_ping: Instant,
}

impl NetClient {
    pub fn connect(server: &str, username: &str) -> std::io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(server)?;
        socket.set_nonblocking(true)?;
        let mut nc = NetClient {
            socket,
            server: server.to_string(),
            username: username.to_string(),
            last_ping: Instant::now(),
        };
        nc.send_connect();
        Ok(nc)
    }

    fn send_connect(&mut self) {
        let msg = format!("CONNECT:{}", self.username);
        let _ = self.socket.send(msg.as_bytes());
    }

    pub fn send_state(&mut self, x: f32, y: f32, angle: f32, level: usize, health: i32, ammo: i32, score: i32) {
        let msg = format!(
            "STATE:{:.1},{:.1},{:.2},{},{},{},{}",
            x, y, angle, level, health, ammo, score
        );
        let _ = self.socket.send(msg.as_bytes());
    }

    pub fn send_disconnect(&mut self) {
        let _ = self.socket.send(b"DISCONNECT");
    }

    pub fn tick(&mut self) -> Vec<String> {
        if self.last_ping.elapsed() > Duration::from_secs(4) {
            let _ = self.socket.send(b"PING");
            self.last_ping = Instant::now();
        }

        let mut buf = [0u8; 2048];
        let mut messages = Vec::new();
        while let Ok((n, _)) = self.socket.recv_from(&mut buf) {
            if n == 0 {
                break;
            }
            let msg = String::from_utf8_lossy(&buf[..n]).to_string();
            messages.push(msg);
        }
        messages
    }
}
