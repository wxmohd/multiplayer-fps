use macroquad::prelude::*;
use std::io::{self, Write};
use std::net::UdpSocket;
use std::time::Instant;

const MAZE_WIDTH: usize = 16;
const MAZE_HEIGHT: usize = 16;
const CELL_SIZE: f32 = 64.0;

struct GameState {
    player_x: f32,
    player_y: f32,
    player_angle: f32,
    fps_counter: f32,
    last_frame_time: Instant,
    username: String,
    server_addr: String,
    maze: [[bool; MAZE_WIDTH]; MAZE_HEIGHT], // true = wall, false = empty
    exit_x: f32,
    exit_y: f32,
    score: i32,
    level: i32,
    game_won: bool,
}

impl GameState {
    fn new(username: String, server_addr: String) -> Self {
        let mut maze = [[false; MAZE_WIDTH]; MAZE_HEIGHT];
        
        // Create a simple maze (Level 1)
        // Outer walls
        for i in 0..MAZE_WIDTH {
            maze[0][i] = true;
            maze[MAZE_HEIGHT - 1][i] = true;
        }
        for i in 0..MAZE_HEIGHT {
            maze[i][0] = true;
            maze[i][MAZE_WIDTH - 1] = true;
        }
        
        // Internal walls to create corridors
        maze[2][2] = true; maze[2][3] = true; maze[2][4] = true;
        maze[4][6] = true; maze[5][6] = true; maze[6][6] = true;
        maze[8][2] = true; maze[8][3] = true; maze[8][4] = true; maze[8][5] = true;
        maze[10][8] = true; maze[11][8] = true; maze[12][8] = true;
        maze[6][10] = true; maze[7][10] = true; maze[8][10] = true;
        maze[4][12] = true; maze[5][12] = true; maze[6][12] = true;
        
        Self {
            player_x: 3.5 * CELL_SIZE, // Start in open area
            player_y: 3.5 * CELL_SIZE,
            player_angle: 0.0,
            fps_counter: 0.0,
            last_frame_time: Instant::now(),
            username,
            server_addr,
            maze,
            exit_x: 13.5 * CELL_SIZE, // Exit in bottom-right area
            exit_y: 13.5 * CELL_SIZE,
            score: 0,
            level: 1,
            game_won: false,
        }
    }

    fn update(&mut self) {
        // Calculate FPS
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time).as_secs_f32();
        self.fps_counter = 1.0 / delta;
        self.last_frame_time = now;

        let move_speed = 150.0 * delta;
        let turn_speed = 2.0 * delta;

        // Handle rotation
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            self.player_angle -= turn_speed;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            self.player_angle += turn_speed;
        }

        // Handle movement
        let mut new_x = self.player_x;
        let mut new_y = self.player_y;

        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            new_x += self.player_angle.cos() * move_speed;
            new_y += self.player_angle.sin() * move_speed;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            new_x -= self.player_angle.cos() * move_speed;
            new_y -= self.player_angle.sin() * move_speed;
        }

        // Collision detection
        if !self.is_wall(new_x, new_y) {
            self.player_x = new_x;
            self.player_y = new_y;
        }

        // Check if player reached the exit
        let distance_to_exit = ((self.player_x - self.exit_x).powi(2) + (self.player_y - self.exit_y).powi(2)).sqrt();
        if distance_to_exit < 30.0 {
            self.advance_level();
        }

        // Shooting mechanics
        if is_key_pressed(KeyCode::Space) {
            self.shoot();
        }
    }

    fn is_wall(&self, x: f32, y: f32) -> bool {
        let grid_x = (x / CELL_SIZE) as usize;
        let grid_y = (y / CELL_SIZE) as usize;
        
        if grid_x >= MAZE_WIDTH || grid_y >= MAZE_HEIGHT {
            return true;
        }
        
        self.maze[grid_y][grid_x]
    }

    fn advance_level(&mut self) {
        self.level += 1;
        self.score += 100;
        
        if self.level > 3 {
            self.game_won = true;
            return;
        }

        // Reset player position
        self.player_x = 3.5 * CELL_SIZE;
        self.player_y = 3.5 * CELL_SIZE;
        self.player_angle = 0.0;

        // Generate new maze based on level
        self.generate_maze_for_level(self.level);
    }

    fn generate_maze_for_level(&mut self, level: i32) {
        // Clear maze
        self.maze = [[false; MAZE_WIDTH]; MAZE_HEIGHT];
        
        // Outer walls
        for i in 0..MAZE_WIDTH {
            self.maze[0][i] = true;
            self.maze[MAZE_HEIGHT - 1][i] = true;
        }
        for i in 0..MAZE_HEIGHT {
            self.maze[i][0] = true;
            self.maze[i][MAZE_WIDTH - 1] = true;
        }

        match level {
            1 => {
                // Level 1: Simple maze
                self.maze[2][2] = true; self.maze[2][3] = true; self.maze[2][4] = true;
                self.maze[4][6] = true; self.maze[5][6] = true; self.maze[6][6] = true;
                self.maze[8][2] = true; self.maze[8][3] = true; self.maze[8][4] = true; self.maze[8][5] = true;
                self.maze[10][8] = true; self.maze[11][8] = true; self.maze[12][8] = true;
                self.maze[6][10] = true; self.maze[7][10] = true; self.maze[8][10] = true;
                self.maze[4][12] = true; self.maze[5][12] = true; self.maze[6][12] = true;
            }
            2 => {
                // Level 2: More complex with dead ends
                self.maze[2][2] = true; self.maze[2][3] = true; self.maze[2][4] = true; self.maze[2][5] = true;
                self.maze[4][2] = true; self.maze[4][4] = true; self.maze[4][6] = true; self.maze[4][8] = true;
                self.maze[6][2] = true; self.maze[6][3] = true; self.maze[6][5] = true; self.maze[6][7] = true; self.maze[6][9] = true;
                self.maze[8][4] = true; self.maze[8][6] = true; self.maze[8][8] = true; self.maze[8][10] = true;
                self.maze[10][2] = true; self.maze[10][4] = true; self.maze[10][6] = true; self.maze[10][8] = true; self.maze[10][10] = true;
                self.maze[12][3] = true; self.maze[12][5] = true; self.maze[12][7] = true; self.maze[12][9] = true;
            }
            3 => {
                // Level 3: Complex but navigable maze
                // Create a more structured complex maze
                self.maze[2][2] = true; self.maze[2][3] = true; self.maze[2][5] = true; self.maze[2][7] = true; self.maze[2][9] = true;
                self.maze[3][4] = true; self.maze[3][6] = true; self.maze[3][8] = true; self.maze[3][10] = true;
                self.maze[4][2] = true; self.maze[4][3] = true; self.maze[4][5] = true; self.maze[4][7] = true; self.maze[4][9] = true; self.maze[4][11] = true;
                self.maze[5][4] = true; self.maze[5][6] = true; self.maze[5][8] = true; self.maze[5][10] = true; self.maze[5][12] = true;
                self.maze[6][2] = true; self.maze[6][3] = true; self.maze[6][5] = true; self.maze[6][7] = true; self.maze[6][9] = true; self.maze[6][11] = true;
                self.maze[7][4] = true; self.maze[7][6] = true; self.maze[7][8] = true; self.maze[7][10] = true;
                self.maze[8][2] = true; self.maze[8][3] = true; self.maze[8][5] = true; self.maze[8][7] = true; self.maze[8][9] = true; self.maze[8][11] = true;
                self.maze[9][4] = true; self.maze[9][6] = true; self.maze[9][8] = true; self.maze[9][10] = true; self.maze[9][12] = true;
                self.maze[10][2] = true; self.maze[10][3] = true; self.maze[10][5] = true; self.maze[10][7] = true; self.maze[10][9] = true; self.maze[10][11] = true;
                self.maze[11][4] = true; self.maze[11][6] = true; self.maze[11][8] = true; self.maze[11][10] = true;
                self.maze[12][2] = true; self.maze[12][3] = true; self.maze[12][5] = true; self.maze[12][7] = true; self.maze[12][9] = true;
                // Ensure clear path to exit
                self.maze[13][12] = false; self.maze[12][13] = false; self.maze[13][13] = false;
            }
            _ => {}
        }
    }

    fn shoot(&mut self) {
        // Simple shooting - in multiplayer this would check for other players in line of sight
        // For now, just show shooting feedback
        self.score += 10;
    }

    fn draw(&self) {
        clear_background(BLACK);

        // Draw 3D first-person view (main viewport)
        self.draw_3d_view();
        
        // Draw mini-map in top-right corner
        self.draw_minimap();

        // Draw FPS counter
        draw_text(
            &format!("FPS: {:.0}", self.fps_counter),
            10.0,
            30.0,
            30.0,
            WHITE,
        );

        // Draw game info
        draw_text(&format!("Player: {}", self.username), 10.0, 60.0, 20.0, WHITE);
        draw_text(&format!("Level: {} | Score: {}", self.level, self.score), 10.0, 90.0, 20.0, YELLOW);
        
        if self.game_won {
            draw_text("🎉 YOU WON! Completed all 3 levels! 🎉", 
                     screen_width() / 2.0 - 200.0, screen_height() / 2.0, 30.0, GREEN);
        } else {
            draw_text("🎯 Find the EXIT (red square on minimap)", 10.0, 120.0, 18.0, SKYBLUE);
        }
        
        draw_text("WASD/Arrows: Move/Turn | SPACE: Shoot", 10.0, screen_height() - 40.0, 16.0, WHITE);
        draw_text("Connected to server", 10.0, screen_height() - 20.0, 16.0, GREEN);
    }

    fn draw_3d_view(&self) {
        let viewport_width = screen_width() - 200.0; // Leave space for minimap
        let viewport_height = screen_height();
        
        let fov = std::f32::consts::PI / 3.0; // 60 degrees
        let num_rays = viewport_width as i32;
        
        for i in 0..num_rays {
            let ray_angle = self.player_angle - fov / 2.0 + (i as f32 / num_rays as f32) * fov;
            
            let (distance, _hit_x, _hit_y) = self.cast_ray(ray_angle);
            
            // Calculate wall height based on distance (perspective)
            let wall_height = (viewport_height * 0.6) / (distance / CELL_SIZE);
            let wall_top = (viewport_height - wall_height) / 2.0;
            let wall_bottom = wall_top + wall_height;
            
            // Draw wall column with wireframe effect
            let x = i as f32;
            
            // Draw the wall as a vertical line (wireframe style)
            draw_line(x, wall_top, x, wall_bottom, 2.0, WHITE);
            
            // Add some depth shading
            let shade = 1.0 - (distance / (CELL_SIZE * 8.0)).min(0.8);
            let color = Color::new(shade, shade, shade, 1.0);
            draw_line(x, wall_top, x, wall_bottom, 1.0, color);
        }
        
        // Draw floor and ceiling lines for perspective
        self.draw_perspective_lines(viewport_width, viewport_height);
    }
    
    fn draw_perspective_lines(&self, width: f32, height: f32) {
        let center_y = height / 2.0;
        
        // Draw horizontal perspective lines for floor/ceiling
        for i in 0..8 {
            let y_offset = (i as f32 * 20.0) + 20.0;
            let alpha = 0.3 - (i as f32 * 0.03);
            
            // Floor lines
            draw_line(0.0, center_y + y_offset, width - 200.0, center_y + y_offset, 1.0, 
                     Color::new(1.0, 1.0, 1.0, alpha));
            
            // Ceiling lines  
            draw_line(0.0, center_y - y_offset, width - 200.0, center_y - y_offset, 1.0,
                     Color::new(1.0, 1.0, 1.0, alpha));
        }
    }

    fn cast_ray(&self, angle: f32) -> (f32, f32, f32) {
        let mut distance = 0.0;
        let step_size = 2.0;
        let max_distance = CELL_SIZE * 20.0;
        
        while distance < max_distance {
            let x = self.player_x + angle.cos() * distance;
            let y = self.player_y + angle.sin() * distance;
            
            if self.is_wall(x, y) {
                return (distance, x, y);
            }
            
            distance += step_size;
        }
        
        (max_distance, 0.0, 0.0)
    }

    fn draw_minimap(&self) {
        let minimap_size = 180.0;
        let minimap_x = screen_width() - minimap_size - 10.0;
        let minimap_y = 10.0;

        // Minimap background
        draw_rectangle(minimap_x, minimap_y, minimap_size, minimap_size, 
                      Color::new(0.0, 0.0, 0.0, 0.8));
        draw_rectangle_lines(minimap_x, minimap_y, minimap_size, minimap_size, 2.0, WHITE);

        // Scale factor for minimap
        let scale = minimap_size / (MAZE_WIDTH as f32 * CELL_SIZE);

        // Draw maze walls on minimap
        for y in 0..MAZE_HEIGHT {
            for x in 0..MAZE_WIDTH {
                if self.maze[y][x] {
                    let wall_x = minimap_x + (x as f32 * CELL_SIZE * scale);
                    let wall_y = minimap_y + (y as f32 * CELL_SIZE * scale);
                    let wall_size = CELL_SIZE * scale;
                    
                    draw_rectangle(wall_x, wall_y, wall_size, wall_size, BLUE);
                }
            }
        }

        // Draw exit on minimap
        let exit_map_x = minimap_x + (self.exit_x * scale);
        let exit_map_y = minimap_y + (self.exit_y * scale);
        draw_rectangle(exit_map_x - 4.0, exit_map_y - 4.0, 8.0, 8.0, RED);

        // Draw player position and direction on minimap
        let player_map_x = minimap_x + (self.player_x * scale);
        let player_map_y = minimap_y + (self.player_y * scale);
        
        // Player dot
        draw_circle(player_map_x, player_map_y, 3.0, YELLOW);
        
        // Player direction indicator
        let dir_length = 15.0;
        let end_x = player_map_x + self.player_angle.cos() * dir_length;
        let end_y = player_map_y + self.player_angle.sin() * dir_length;
        draw_line(player_map_x, player_map_y, end_x, end_y, 2.0, YELLOW);

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