use macroquad::prelude::*;
use std::io::{self, Write};
use std::net::UdpSocket;
use std::time::Instant;

struct GameState {
    player_x: f32,
    player_y: f32,
    camera_x: f32,
    camera_y: f32,
    fps_counter: f32,
    last_frame_time: Instant,
    username: String,
    server_addr: String,
}

impl GameState {
    fn new(username: String, server_addr: String) -> Self {
        Self {
            player_x: 400.0,
            player_y: 300.0,
            camera_x: 0.0,
            camera_y: 0.0,
            fps_counter: 0.0,
            last_frame_time: Instant::now(),
            username,
            server_addr,
        }
    }

    fn update(&mut self) {
        // Calculate FPS
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time).as_secs_f32();
        self.fps_counter = 1.0 / delta;
        self.last_frame_time = now;

        // Handle input
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            self.player_y -= 200.0 * delta;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            self.player_y += 200.0 * delta;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            self.player_x -= 200.0 * delta;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            self.player_x += 200.0 * delta;
        }

        // Update camera to follow player
        self.camera_x = self.player_x - screen_width() / 2.0;
        self.camera_y = self.player_y - screen_height() / 2.0;
    }

    fn draw(&self) {
        clear_background(DARKGRAY);

        // Draw world (simple maze walls)
        self.draw_world();
        
        // Draw player
        draw_circle(
            self.player_x - self.camera_x,
            self.player_y - self.camera_y,
            10.0,
            RED,
        );

        // Draw mini-map
        self.draw_minimap();

        // Draw FPS counter
        draw_text(
            &format!("FPS: {:.0}", self.fps_counter),
            10.0,
            30.0,
            30.0,
            WHITE,
        );

        // Draw UI
        draw_text(&format!("Player: {}", self.username), 10.0, 60.0, 20.0, WHITE);
        draw_text(&format!("Server: {}", self.server_addr), 10.0, 80.0, 20.0, WHITE);
        draw_text("Use WASD or Arrow keys to move", 10.0, screen_height() - 20.0, 20.0, WHITE);
    }

    fn draw_world(&self) {
        // Simple maze walls for now
        let walls = [
            (100.0, 100.0, 600.0, 20.0), // Top wall
            (100.0, 500.0, 600.0, 20.0), // Bottom wall
            (100.0, 100.0, 20.0, 400.0), // Left wall
            (680.0, 100.0, 20.0, 400.0), // Right wall
            (300.0, 200.0, 20.0, 200.0), // Internal wall 1
            (500.0, 300.0, 200.0, 20.0), // Internal wall 2
        ];

        for (x, y, w, h) in walls {
            draw_rectangle(
                x - self.camera_x,
                y - self.camera_y,
                w,
                h,
                BLUE,
            );
        }
    }

    fn draw_minimap(&self) {
        let minimap_size = 150.0;
        let minimap_x = screen_width() - minimap_size - 10.0;
        let minimap_y = 10.0;

        // Minimap background
        draw_rectangle(minimap_x, minimap_y, minimap_size, minimap_size, Color::new(0.0, 0.0, 0.0, 0.7));
        draw_rectangle_lines(minimap_x, minimap_y, minimap_size, minimap_size, 2.0, WHITE);

        // Scale factor for minimap
        let scale = minimap_size / 800.0;

        // Draw walls on minimap
        let walls = [
            (100.0, 100.0, 600.0, 20.0),
            (100.0, 500.0, 600.0, 20.0),
            (100.0, 100.0, 20.0, 400.0),
            (680.0, 100.0, 20.0, 400.0),
            (300.0, 200.0, 20.0, 200.0),
            (500.0, 300.0, 200.0, 20.0),
        ];

        for (x, y, w, h) in walls {
            draw_rectangle(
                minimap_x + x * scale,
                minimap_y + y * scale,
                w * scale,
                h * scale,
                BLUE,
            );
        }

        // Draw player position on minimap
        draw_circle(
            minimap_x + self.player_x * scale,
            minimap_y + self.player_y * scale,
            3.0,
            RED,
        );

        // Minimap label
        draw_text("Mini-map", minimap_x, minimap_y - 5.0, 16.0, WHITE);
    }
}

fn get_user_input() -> Result<(String, String), Box<dyn std::error::Error>> {
    println!("=== Multiplayer FPS Client ===");
    
    // Prompt for server IP
    print!("Enter IP Address: ");
    io::stdout().flush().unwrap();
    let mut ip_input = String::new();
    io::stdin().read_line(&mut ip_input).unwrap();
    let server_ip = ip_input.trim();
    
    // Default to localhost if empty
    let server_addr = if server_ip.is_empty() {
        "127.0.0.1:34254".to_string()
    } else {
        server_ip.to_string()
    };
    
    // Prompt for username
    print!("Enter Name: ");
    io::stdout().flush().unwrap();
    let mut name_input = String::new();
    io::stdin().read_line(&mut name_input).unwrap();
    let username = name_input.trim().to_string();
    
    println!("Starting...");
    println!("Connecting to server: {}", server_addr);
    println!("Username: {}", username);
    
    Ok((username, server_addr))
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Multiplayer FPS".to_owned(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (username, server_addr) = get_user_input()?;
    
    // Try to connect to server
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => {
            if let Ok(_) = socket.connect(&server_addr) {
                let connect_msg = format!("CONNECT:{}", username);
                let _ = socket.send(connect_msg.as_bytes());
                println!("Connected to server!");
            } else {
                println!("Warning: Could not connect to server, running in offline mode");
            }
        }
        Err(_) => {
            println!("Warning: Could not create socket, running in offline mode");
        }
    }

    let mut game_state = GameState::new(username, server_addr);

    loop {
        game_state.update();
        game_state.draw();
        next_frame().await;
    }
}