use macroquad::prelude::*;
// use std::collections::HashMap;
// use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use std::io::{self, Write};

mod renderer;
mod assets;
mod sprite_gen;
mod multiplayer;

use renderer::Renderer;
use assets::AssetManager;
use multiplayer::MultiplayerManager;

// ----------------- constants -----------------
const MAX_MAZE_WIDTH: usize = 40;
const MAX_MAZE_HEIGHT: usize = 40;
const CELL_SIZE: f32 = 32.0;
const WALL_HEIGHT: f32 = 64.0;
const FOV: f32 = std::f32::consts::PI / 3.0;
const RENDER_DISTANCE: f32 = 1000.0;
const TARGET_FPS: u32 = 60;

#[derive(Clone)]
struct OtherPlayer {
    name: String,
    x: f32,
    y: f32,
    angle: f32,
    health: i32,
    level: i32,
    last_seen: Instant,
}

struct Bullet {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    owner: String,
    created: Instant,
}

#[derive(Clone)]
struct Notification {
    message: String,
    color: Color,
    created: Instant,
}

// ----------------- game state -----------------
struct GameState {
    // player
    player_x: f32,
    player_y: f32,
    player_angle: f32,

    // world
    maze: [[bool; MAX_MAZE_WIDTH]; MAX_MAZE_HEIGHT],
    maze_width: usize,
    maze_height: usize,
    level: usize,
    exit_x: f32,
    exit_y: f32,

    // connection (echo/offline ok)
    server_addr: String,
    username: String,

    // input
    mouse_sensitivity: f32,
    last_mouse_x: f32,

    // perf
    frame_times: Vec<f32>,
    last_frame_time: Instant,
    fps_counter: f32,

    // gameplay
    health: i32,
    ammo: i32,
    score: i32,
    game_won: bool,

    // fx
    crosshair_pulse: f32,
    wall_hit_flash: f32,

    // rendering
    renderer: Renderer,

    // assets
    assets: AssetManager,

    // viewmodel (weapon) – screen-space
    gun_fire_t: f32,     // seconds left in muzzle flash
    gun_recoil: f32,     // 0..1 recoil kick
    gun_bob_phase: f32,  // walk/breath bob phase
    
    // multiplayer
    multiplayer: MultiplayerManager,
}

impl GameState {
    fn new(username: String, server_addr: String) -> Self {
        // Initialize maze dimensions based on level
        let (maze_width, maze_height) = match 1 {
            1 => (20, 20),
            2 => (28, 28), 
            3 => (36, 36),
            _ => (20, 20),
        };
        
        // simple perimeter maze with some columns
        let mut maze = [[false; MAX_MAZE_WIDTH]; MAX_MAZE_HEIGHT];
        for x in 0..maze_width {
            maze[0][x] = true;
            maze[maze_height - 1][x] = true;
        }
        for y in 0..maze_height {
            maze[y][0] = true;
            maze[y][maze_width - 1] = true;
        }
        maze[2][2] = true;
        maze[2][3] = true;
        maze[2][4] = true;
        maze[4][6] = true;
        maze[5][6] = true;
        maze[6][6] = true;
        maze[8][2] = true;
        maze[8][3] = true;
        maze[8][4] = true;
        maze[8][5] = true;

        Self {
            // player
            player_x: 3.5 * CELL_SIZE,
            player_y: 3.5 * CELL_SIZE,
            player_angle: 0.0,

            // world
            maze,
            maze_width,
            maze_height,
            level: 1,
            exit_x: (maze_width - 3) as f32 * CELL_SIZE + CELL_SIZE * 0.5,
            exit_y: (maze_height - 3) as f32 * CELL_SIZE + CELL_SIZE * 0.5,

            // net
            server_addr,
            username,

            // input
            mouse_sensitivity: 0.003,
            last_mouse_x: 0.0,

            // perf
            frame_times: Vec::with_capacity(60),
            last_frame_time: Instant::now(),
            fps_counter: 60.0,

            // gameplay
            health: 100,
            ammo: 30,
            score: 0,
            game_won: false,

            // fx
            crosshair_pulse: 0.0,
            wall_hit_flash: 0.0,

            // rendering
            renderer: Renderer::new(),

            // assets
            assets: AssetManager::new(),

            // weapon
            gun_fire_t: 0.0,
            gun_recoil: 0.0,
            gun_bob_phase: 0.0,
            
            // multiplayer
            multiplayer: MultiplayerManager::new(),
        }
    }

    fn is_wall(&self, x: f32, y: f32) -> bool {
        let gx = (x / CELL_SIZE) as usize;
        let gy = (y / CELL_SIZE) as usize;
        if gx >= self.maze_width || gy >= self.maze_height {
            return true;
        }
        self.maze[gy][gx]
    }

    fn advance_level(&mut self) {
        self.level += 1;
        self.score += 100;
        if self.level > 3 {
            self.game_won = true;
            return;
        }
        
        // Update maze dimensions based on level
        let (new_width, new_height) = match self.level {
            1 => (20, 20),
            2 => (28, 28), 
            3 => (36, 36),
            _ => (20, 20),
        };
        self.maze_width = new_width;
        self.maze_height = new_height;
        
        self.player_x = 3.5 * CELL_SIZE;
        self.player_y = 3.5 * CELL_SIZE;
        self.player_angle = 0.0;
        
        // Update exit position based on new maze size
        self.exit_x = (self.maze_width - 3) as f32 * CELL_SIZE + CELL_SIZE * 0.5;
        self.exit_y = (self.maze_height - 3) as f32 * CELL_SIZE + CELL_SIZE * 0.5;
        
        self.generate_dense_maze_pattern();
        
        // Ensure player starting area is clear
        for dy in -1..=1 {
            for dx in -1..=1 {
                let px = 3 + dx;
                let py = 3 + dy;
                if px >= 0 && px < self.maze_width as i32 && py >= 0 && py < self.maze_height as i32 {
                    self.maze[py as usize][px as usize] = false;
                }
            }
        }
        
        // Create dense maze with creative patterns - much more challenging
        self.generate_dense_maze_pattern();
        
        // Add more dead ends and false paths based on level
        let false_path_density = match self.level {
            1 => 0.60, // Much higher density of false paths
            2 => 0.75, // Very high density
            3 => 0.85, // Extremely high density - almost every open space becomes a dead end
            _ => 0.50,
        };
        
        for y in 3..(self.maze_height - 3) {
            for x in 3..(self.maze_width - 3) {
                if !self.maze[y][x] {
                    let noise = ((x * 11 + y * 17 + self.level as usize * 23) % 100) as f32 / 100.0;
                    if noise < false_path_density {
                        // Create longer, more complex dead end networks
                        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
                        let dir_idx = (x + y + self.level as usize) % directions.len();
                        let (dx, dy) = directions[dir_idx];
                        
                        // Create longer dead end paths based on level
                        let path_length = match self.level {
                            1 => 4, // Longer paths
                            2 => 6, // Even longer paths
                            3 => 8, // Very long winding dead ends
                            _ => 3,
                        };
                        
                        for i in 1..=path_length {
                            let nx = x as i32 + dx * i;
                            let ny = y as i32 + dy * i;
                            if nx > 0 && nx < (self.maze_width - 1) as i32 && 
                               ny > 0 && ny < (self.maze_height - 1) as i32 {
                                self.maze[ny as usize][nx as usize] = false;
                            }
                        }
                        
                        // Add branching dead ends for higher levels
                        if self.level >= 2 {
                            let perpendicular = [(-dy, dx), (dy, -dx)];
                            for &(pdx, pdy) in &perpendicular {
                                for i in 1..=3 {
                                    let branch_x = x as i32 + dx * 2 + pdx * i;
                                    let branch_y = y as i32 + dy * 2 + pdy * i;
                                    if branch_x > 0 && branch_x < (self.maze_width - 1) as i32 && 
                                       branch_y > 0 && branch_y < (self.maze_height - 1) as i32 {
                                        self.maze[branch_y as usize][branch_x as usize] = false;
                                    }
                                }
                            }
                        }
                        
                        // Block the end to create dead end
                        let end_x = x as i32 + dx * (path_length + 1);
                        let end_y = y as i32 + dy * (path_length + 1);
                        if end_x > 0 && end_x < (self.maze_width - 1) as i32 && 
                           end_y > 0 && end_y < (self.maze_height - 1) as i32 {
                            self.maze[end_y as usize][end_x as usize] = true;
                        }
                    }
                }
            }
        }
        
        // Ensure clear path to exit exists
        let exit_x = self.maze_width - 3;
        let exit_y = self.maze_height - 3;
        self.maze[exit_y][exit_x] = false;
        
        // Create guaranteed path from start to exit
        let mut path_x = 3;
        let mut path_y = 3;
        
        // Horizontal path first
        while path_x < exit_x {
            self.maze[path_y][path_x] = false;
            self.maze[path_y + 1][path_x] = false; // Make corridor wider
            path_x += 1;
        }
        
        // Vertical path
        while path_y < exit_y {
            self.maze[path_y][path_x] = false;
            self.maze[path_y][path_x - 1] = false; // Make corridor wider
            path_y += 1;
        }
        
        // Ensure exit area is clear
        for dy in -1..=1 {
            for dx in -1..=1 {
                let ex = exit_x as i32 + dx;
                let ey = exit_y as i32 + dy;
                if ex >= 1 && ex < (self.maze_width - 1) as i32 && ey >= 1 && ey < (self.maze_height - 1) as i32 {
                    self.maze[ey as usize][ex as usize] = false;
                }
            }
        }

        self.health = 100;
        self.ammo = 30;
    }

    fn shoot(&mut self) {
        if self.ammo <= 0 {
            return;
        }
        self.ammo -= 1;
        self.wall_hit_flash = 0.25;

        // weapon feedback
        self.gun_fire_t = 0.12; // 120 ms flash
        self.gun_recoil = 1.0;  // max kick
    }

    fn update(&mut self) {
        // --- FPS / delta ---
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time).as_secs_f32();
        
        // --- Multiplayer networking ---
        if let Some((spawn_x, spawn_y)) = self.multiplayer.handle_network_messages() {
            self.player_x = spawn_x;
            self.player_y = spawn_y;
        }
        self.multiplayer.send_player_state(self.player_x, self.player_y, self.player_angle, self.level as i32, self.health);
        
        // Debug: Print local player position and other players
        if self.multiplayer.other_players.len() > 0 {
            println!("🎮 Local player: ({:.1}, {:.1}) | Other players: {}", 
                    self.player_x, self.player_y, self.multiplayer.other_players.len());
            for (name, player) in &self.multiplayer.other_players {
                println!("  👤 {}: ({:.1}, {:.1})", name, player.x, player.y);
            }
        }
        let (hit_player, hit_by) = self.multiplayer.update_bullets(delta, self.player_x, self.player_y, &self.username, &self.maze, self.maze_width, self.maze_height);
        if hit_player {
            self.health -= 20;
            if let Some(shooter) = hit_by {
                self.multiplayer.show_notification(&format!("Hit by {}!", shooter), RED);
            }
            if self.health <= 0 {
                self.health = 100;
                self.multiplayer.deaths += 1;
                let (spawn_x, spawn_y) = self.multiplayer.get_random_spawn_point(&self.maze, self.maze_width, self.maze_height);
                self.player_x = spawn_x;
                self.player_y = spawn_y;
                self.multiplayer.show_notification("You were eliminated! Respawning...", ORANGE);
            }
        }
        self.last_frame_time = now;
        let fps_now = 1.0 / delta.max(0.001);
        self.frame_times.push(fps_now);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }
        self.fps_counter = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;

        // --- simple time values ---
        self.crosshair_pulse += delta * 3.0;
        self.wall_hit_flash = (self.wall_hit_flash - delta * 2.0).max(0.0);

        // --- mouse look ---
        let (mx, _) = mouse_position();
        if self.last_mouse_x != 0.0 {
            self.player_angle += (mx - self.last_mouse_x) * self.mouse_sensitivity;
        }
        self.last_mouse_x = mx;

        // --- movement ---
        let move_speed = 200.0 * delta;
        let strafe_speed = 180.0 * delta;
        let mut nx = self.player_x;
        let mut ny = self.player_y;

        if is_key_down(KeyCode::W) {
            nx += self.player_angle.cos() * move_speed;
            ny += self.player_angle.sin() * move_speed;
        }
        if is_key_down(KeyCode::S) {
            nx -= self.player_angle.cos() * move_speed;
            ny -= self.player_angle.sin() * move_speed;
        }
        // Arrow keys: Up/Down move forward/back just like W/S
        if is_key_down(KeyCode::Up) {
            nx += self.player_angle.cos() * move_speed;
            ny += self.player_angle.sin() * move_speed;
        }
        if is_key_down(KeyCode::Down) {
            nx -= self.player_angle.cos() * move_speed;
            ny -= self.player_angle.sin() * move_speed;
        }
        if is_key_down(KeyCode::A) {
            nx += (self.player_angle - std::f32::consts::PI / 2.0).cos() * strafe_speed;
            ny += (self.player_angle - std::f32::consts::PI / 2.0).sin() * strafe_speed;
        }
        if is_key_down(KeyCode::D) {
            nx += (self.player_angle + std::f32::consts::PI / 2.0).cos() * strafe_speed;
            ny += (self.player_angle + std::f32::consts::PI / 2.0).sin() * strafe_speed;
        }
        if is_key_down(KeyCode::Left) {
            self.player_angle -= 2.0 * delta;
        }
        if is_key_down(KeyCode::Right) {
            self.player_angle += 2.0 * delta;
        }

        if !self.is_wall(nx, ny) {
            self.player_x = nx;
            self.player_y = ny;
        } else {
            self.wall_hit_flash = 0.3;
        }

        // Theme switching removed - game maintains consistent design throughout

        // exit trigger
        let d_exit = ((self.player_x - self.exit_x).powi(2) + (self.player_y - self.exit_y).powi(2)).sqrt();
        if d_exit < 40.0 {
            self.advance_level();
        }

        // shooting
        if is_key_pressed(KeyCode::Space) {
            self.shoot_bullet();
        }
        
        // ammo recharge
        if is_key_pressed(KeyCode::R) {
            self.recharge_ammo();
        }

        // --- weapon timers ---
        // Drive weapon bob only from actual movement (not rotation): WASD + Up/Down
        let moving = is_key_down(KeyCode::W)
            || is_key_down(KeyCode::A)
            || is_key_down(KeyCode::S)
            || is_key_down(KeyCode::D)
            || is_key_down(KeyCode::Up)
            || is_key_down(KeyCode::Down);

        if moving {
            self.gun_bob_phase += delta * 6.0;
        }
        self.gun_fire_t = (self.gun_fire_t - delta).max(0.0);
        self.gun_recoil = (self.gun_recoil - 3.2 * delta).max(0.0);
    }

    fn draw(&mut self) {
        self.renderer.update_fps();
        self.draw_3d_view();
        self.draw_minimap();
        self.renderer.draw_hud(self.level as u32, self.health, self.score, self.ammo);
        
        
        // Victory screen when all levels completed
        if self.game_won {
            self.draw_victory_screen();
        }
        
        // Draw notifications and score
        self.multiplayer.draw_notifications();
        self.draw_score_display();
    }

    
    fn draw_victory_screen(&self) {
        let sw = screen_width();
        let sh = screen_height();
        
        // Dark overlay
        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(0, 0, 0, 200));
        
        // Victory message
        let title = "🎉 CONGRATULATIONS! 🎉";
        let subtitle = "You have conquered all 3 levels!";
        let instruction = "Press ESC to exit";
        
        // Title
        let title_size = 48.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(title, (sw - title_width) / 2.0, sh / 2.0 - 100.0, title_size, GOLD);
        
        // Subtitle
        let subtitle_size = 32.0;
        let subtitle_width = measure_text(subtitle, None, subtitle_size as u16, 1.0).width;
        draw_text(subtitle, (sw - subtitle_width) / 2.0, sh / 2.0 - 40.0, subtitle_size, WHITE);
        
        // Instruction
        let inst_size = 20.0;
        let inst_width = measure_text(instruction, None, inst_size as u16, 1.0).width;
        draw_text(instruction, (sw - inst_width) / 2.0, sh / 2.0 + 80.0, inst_size, YELLOW);
        
        // Animated border
        let pulse = (get_time() * 3.0).sin() * 0.3 + 0.7;
        let border_color = Color::new(1.0, pulse as f32, 0.0, pulse as f32);
        draw_rectangle_lines(50.0, sh / 2.0 - 150.0, sw - 100.0, 280.0, 4.0, border_color);
    }
    
    fn draw_other_players(&self) {
        for player in self.multiplayer.other_players.values() {
            // Only render players in the same level
            if player.level != self.level as i32 {
                continue;
            }
            // Calculate relative position in world space
            let world_dx = player.x - self.player_x;
            let world_dy = player.y - self.player_y;
            let distance = (world_dx * world_dx + world_dy * world_dy).sqrt();
            
            // Convert to view space: rotate by player's viewing angle
            let view_forward_x = self.player_angle.cos();
            let view_forward_y = self.player_angle.sin();
            let view_right_x = -view_forward_y;
            let view_right_y = view_forward_x;
            
            // Project world offset onto view axes
            let view_x = world_dx * view_right_x + world_dy * view_right_y;
            let view_z = world_dx * view_forward_x + world_dy * view_forward_y;
            
            // Debug output
            println!("🔍 Player {}: world_offset=({:.1}, {:.1}), view_space=({:.1}, {:.1})", 
                    player.name, world_dx, world_dy, view_x, view_z);
            
            // Only draw if in front and within range
            if distance < 1200.0 && view_z > 20.0 {
                // Perspective projection
                let screen_scale = 500.0 / view_z;
                let screen_x = screen_width() / 2.0 + view_x * screen_scale;
                let screen_y = screen_height() / 2.0;
                
                // Ensure it's on screen
                if screen_x > -50.0 && screen_x < screen_width() + 50.0 {
                    // Draw other player as a bright colored rectangle
                    let player_color = if player.health > 50 { GREEN } else if player.health > 25 { YELLOW } else { RED };
                    let size = (60.0 * screen_scale).clamp(15.0, 120.0);
                    
                    // Draw player body
                    draw_rectangle(screen_x - size / 2.0, screen_y - size, size, size, player_color);
                    
                    // Draw outline for visibility
                    draw_rectangle_lines(screen_x - size / 2.0, screen_y - size, size, size, 2.0, WHITE);
                    
                    // Draw player name above
                    let name_width = measure_text(&player.name, None, 18, 1.0).width;
                    draw_text(&player.name, screen_x - name_width / 2.0, screen_y - size - 15.0, 18.0, WHITE);
                    
                    // Draw health bar
                    let health_width = size;
                    let health_height = 4.0;
                    let health_percent = player.health as f32 / 100.0;
                    draw_rectangle(screen_x - health_width / 2.0, screen_y - size - 25.0, health_width, health_height, DARKGRAY);
                    draw_rectangle(screen_x - health_width / 2.0, screen_y - size - 25.0, health_width * health_percent, health_height, player_color);
                }
            }
        }
    }
    
    fn draw_bullets(&self) {
        for bullet in &self.multiplayer.bullets {
            // Calculate screen position relative to current player
            let dx = bullet.x - self.player_x;
            let dy = bullet.y - self.player_y;
            
            // Rotate relative to player's view angle
            let cos_a = self.player_angle.cos();
            let sin_a = self.player_angle.sin();
            let rx = dx * cos_a + dy * sin_a;
            let ry = -dx * sin_a + dy * cos_a;
            
            // Only draw if in front of player
            if ry > 0.0 {
                let distance = (dx * dx + dy * dy).sqrt();
                if distance < RENDER_DISTANCE {
                    // Project to screen
                    let screen_x = screen_width() / 2.0 + rx * (screen_width() / 2.0) / ry;
                    let screen_y = screen_height() / 2.0;
                    
                    // Draw bullet as a bright dot
                    let bullet_color = if bullet.owner == self.username {
                        BLUE
                    } else {
                        RED
                    };
                    
                    draw_circle(screen_x, screen_y, 3.0, bullet_color);
                }
            }
        }
    }

    fn draw_3d_view(&self) {
        let sw = screen_width();
        let sh = screen_height();

        clear_background(BLACK);

        // ceiling
        draw_rectangle(0.0, 0.0, sw, sh / 2.0, Renderer::get_sky_color(self.level as u32));
        // floor
        draw_rectangle(0.0, sh / 2.0, sw, sh / 2.0, Renderer::get_floor_color(self.level as u32));

        // raycast walls as vertical columns
        let rays = 320;
        for i in 0..rays {
            let ray_angle = self.player_angle - FOV / 2.0 + (i as f32 / rays as f32) * FOV;

            // cast
            let mut d = 0.0f32;
            let rc = ray_angle.cos();
            let rs = ray_angle.sin();
            while d < RENDER_DISTANCE {
                let tx = self.player_x + rc * d;
                let ty = self.player_y + rs * d;
                if self.is_wall(tx, ty) {
                    break;
                }
                d += 1.0;
            }

            // fish-eye fix
            d *= (ray_angle - self.player_angle).cos();

            // projected column - make walls much taller for immersive maze experience
            let wall_h = (sh * 2.5) / (d / CELL_SIZE + 0.1);
            let wall_top = (sh / 2.0) - wall_h / 2.0;
            let wall_bottom = wall_top + wall_h;

            let _brightness = 1.0; // constant (no distance darken, per request)
            
            // Draw realistic textured walls instead of plain blocks
            let x = (i as f32 / rays as f32) * sw;
            let w = (sw / rays as f32).max(1.0);
            self.draw_textured_wall_column(x, wall_top, w, wall_bottom - wall_top, self.level as u32, i);
        }

        // Draw 3D exit visualization
        self.draw_3d_exit_door();
        
        // Draw other players and bullets
        self.draw_other_players();
        self.draw_bullets();
        
        // Enhanced exit proximity indicator with arrows and text
        let dx = self.exit_x - self.player_x;
        let dy = self.exit_y - self.player_y;
        let dist = (dx * dx + dy * dy).sqrt();
        
        // Show different indicators based on distance (only when very close)
        if dist < 150.0 {
            let angle_to_exit = dy.atan2(dx);
            let mut rel = angle_to_exit - self.player_angle;
            
            // Normalize angle
            while rel > std::f32::consts::PI {
                rel -= 2.0 * std::f32::consts::PI;
            }
            while rel < -std::f32::consts::PI {
                rel += 2.0 * std::f32::consts::PI;
            }
            
            // Direction arrow and text when exit is in view
            if rel.abs() < 0.6 {
                let pulse = (get_time() * 6.0).sin() * 0.3 + 0.7;
                let intensity = (150.0 - dist) / 150.0; // Stronger as you get closer
                let col = Color::new(1.0, pulse as f32 * intensity, 0.0, intensity);
                
                // Large directional arrow pointing to exit
                let arrow_x = sw / 2.0 + rel * 150.0;
                let arrow_y = sh / 2.0 - 80.0;
                
                // Draw arrow shape
                let arrow_size = 8.0 + pulse as f32 * 6.0;
                draw_triangle(
                    Vec2::new(arrow_x, arrow_y - arrow_size),
                    Vec2::new(arrow_x - arrow_size * 0.7, arrow_y + arrow_size * 0.5),
                    Vec2::new(arrow_x + arrow_size * 0.7, arrow_y + arrow_size * 0.5),
                    col
                );
                
                // Exit text with level-specific theming
                let exit_text = match self.level {
                    1 => "🏛️ ARCH EXIT",
                    2 => "🌀 PORTAL EXIT", 
                    3 => "✨ CRYSTAL GATE",
                    _ => "🚪 EXIT"
                };
                
                let text_size = 18.0 + pulse as f32 * 4.0;
                draw_text(exit_text, arrow_x - 50.0, arrow_y + 30.0, text_size, col);
                
                // Distance text
                let dist_text = format!("Distance: {:.1}m", dist / CELL_SIZE);
                draw_text(&dist_text, sw / 2.0 - 60.0, sh - 80.0, 16.0, col);
                
                // Approach instruction when very close
                if dist < 80.0 {
                    let approach_text = ">>> WALK INTO EXIT <<<";
                    let approach_col = Color::new(1.0, 1.0, 0.0, pulse as f32);
                    draw_text(approach_text, sw / 2.0 - 100.0, sh / 2.0 + 100.0, 20.0, approach_col);
                }
            }
            
            // Compass-style indicator when exit is not in direct view
            else {
                let pulse = (get_time() * 4.0).sin() * 0.2 + 0.8;
                let intensity = (150.0 - dist) / 150.0;
                let col = Color::new(1.0, 0.5, 0.0, intensity * pulse as f32);
                
                // Compass arrow at edge of screen
                let compass_x = sw / 2.0 + rel.signum() * (sw / 2.0 - 60.0);
                let compass_y = sh / 2.0;
                
                // Draw compass arrow
                draw_triangle(
                    Vec2::new(compass_x + rel.signum() * 10.0, compass_y),
                    Vec2::new(compass_x - rel.signum() * 5.0, compass_y - 8.0),
                    Vec2::new(compass_x - rel.signum() * 5.0, compass_y + 8.0),
                    col
                );
                
                draw_text("EXIT", compass_x - 15.0, compass_y + 25.0, 16.0, col);
                let dist_text = format!("{:.0}m", dist / CELL_SIZE);
                draw_text(&dist_text, compass_x - 10.0, compass_y + 40.0, 14.0, col);
            }
        }

        // hit flash vignette
        if self.wall_hit_flash > 0.0 {
            let a = (self.wall_hit_flash * 100.0) as u8;
            draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(255, 60, 60, a));
        }

        // crosshair
        let cx = sw / 2.0;
        let cy = sh / 2.0;
        let s = 12.0;
        let th = 2.0;
        let c = WHITE;
        draw_line(cx - s, cy, cx - 4.0, cy, th, c);
        draw_line(cx + 4.0, cy, cx + s, cy, th, c);
        draw_line(cx, cy - s, cx, cy - 4.0, th, c);
        draw_line(cx, cy + 4.0, cx, cy + s, th, c);
        draw_circle(cx, cy, 1.5, c);

        // screen-space weapon (viewmodel) — faces forward
        Renderer::draw_weapon(
            self.level as u32,
            self.gun_fire_t,
            self.gun_recoil,
            self.gun_bob_phase,
            self.ammo,
            &self.assets,
        );
    }

    fn draw_minimap(&self) {
        let size = 180.0;
        let mx = screen_width() - size - 10.0;
        let my = 10.0;
        let cell = size / self.maze_width as f32;

        // Enhanced minimap background with modern level-themed colors
        let bg_color = match self.level {
            1 => Color::from_rgba(245, 245, 240, 120),   // Clean white stone
            2 => Color::from_rgba(45, 55, 70, 120),      // Modern tech blue-gray
            3 => Color::from_rgba(240, 248, 255, 120),   // Elegant crystal blue
            _ => Color::from_rgba(0, 0, 30, 120),
        };
        
        draw_rectangle(mx - 5.0, my - 25.0, size + 10.0, size + 30.0, Color::from_rgba(0, 0, 0, 200));
        draw_rectangle(mx, my, size, size, bg_color);
        draw_rectangle_lines(mx, my, size, size, 3.0, WHITE);
        
        // Title with player count
        let player_count_text = format!("MAZE MAP - {} PLAYERS", self.multiplayer.get_player_count());
        draw_text(&player_count_text, mx + 5.0, my - 8.0, 16.0, WHITE);

        // Draw maze walls with accurate scaling
        for y in 0..self.maze_height {
            for x in 0..self.maze_width {
                if self.maze[y][x] {
                    let wall_color = match self.level {
                        1 => Color::from_rgba(160, 82, 45, 255),   // Saddle brown
                        2 => Color::from_rgba(70, 130, 180, 255),  // Steel blue
                        3 => Color::from_rgba(147, 112, 219, 255), // Medium purple
                        _ => Color::from_rgba(100, 150, 255, 255),
                    };
                    draw_rectangle(
                        mx + x as f32 * cell,
                        my + y as f32 * cell,
                        cell,
                        cell,
                        wall_color,
                    );
                }
            }
        }

        // Enhanced themed exit indicator with distance-based effects
        let distance_to_exit = ((self.player_x - self.exit_x).powi(2) + (self.player_y - self.exit_y).powi(2)).sqrt();
        let proximity_factor = (1.0 - (distance_to_exit / 800.0).min(1.0)).max(0.0);
        let pulse = (get_time() * (3.0 + proximity_factor as f64 * 4.0)).sin() * 0.4 + 0.6;
        let glow_intensity = pulse * (0.5 + proximity_factor as f64 * 0.5);
        
        let ex = mx + (self.exit_x / CELL_SIZE) * cell;
        let ey = my + (self.exit_y / CELL_SIZE) * cell;
        
        match self.level {
            1 => {
                // Golden architectural exit with expanding glow when close
                let glow_size = cell * (1.0 + proximity_factor * 0.8);
                draw_rectangle(ex - (glow_size - cell) * 0.5, ey - (glow_size - cell) * 0.5, 
                              glow_size, glow_size, Color::from_rgba(255, 215, 0, (glow_intensity * 100.0) as u8));
                draw_rectangle(ex, ey, cell, cell, Color::from_rgba(255, 215, 0, (glow_intensity * 200.0) as u8));
                draw_text("ARCH", ex - 8.0, ey - 5.0, 10.0, Color::from_rgba(255, 255, 255, 255));
            },
            2 => {
                // Pulsing tech portal with energy rings
                for ring in 0..3 {
                    let ring_size = cell * (1.2 + ring as f32 * 0.3 + proximity_factor * 0.5);
                    let ring_alpha = (glow_intensity * 80.0 / (ring + 1) as f64) as u8;
                    draw_rectangle(ex - (ring_size - cell) * 0.5, ey - (ring_size - cell) * 0.5,
                                  ring_size, ring_size, Color::from_rgba(0, 150, 255, ring_alpha));
                }
                draw_rectangle(ex, ey, cell, cell, Color::from_rgba(0, 200, 255, (glow_intensity * 220.0) as u8));
                draw_text("PORTAL", ex - 12.0, ey - 5.0, 9.0, Color::from_rgba(150, 220, 255, 255));
            },
            3 => {
                // Radiant crystal gateway with prismatic effects
                let crystal_glow = cell * (1.0 + proximity_factor * 1.2);
                // Multiple colored layers for prismatic effect
                let colors = [
                    Color::from_rgba(255, 255, 255, (glow_intensity * 60.0) as u8),
                    Color::from_rgba(200, 220, 255, (glow_intensity * 80.0) as u8),
                    Color::from_rgba(255, 200, 255, (glow_intensity * 60.0) as u8),
                ];
                for (i, color) in colors.iter().enumerate() {
                    let layer_size = crystal_glow * (1.0 - i as f32 * 0.2);
                    draw_rectangle(ex - (layer_size - cell) * 0.5, ey - (layer_size - cell) * 0.5,
                                  layer_size, layer_size, *color);
                }
                draw_rectangle(ex, ey, cell, cell, Color::from_rgba(255, 255, 255, (glow_intensity * 240.0) as u8));
                draw_text("GATE", ex - 8.0, ey - 5.0, 10.0, Color::from_rgba(200, 220, 255, 255));
            },
            _ => {
                draw_rectangle(ex, ey, cell, cell, Color::new(1.0, pulse as f32 * 0.5, 0.0, 1.0));
                draw_text("EXIT", ex - 8.0, ey - 5.0, 12.0, WHITE);
            }
        }

        // Player position - accurate positioning with cell centering
        // Your player dot (larger and distinctive)
        draw_circle(mx + (self.player_x / CELL_SIZE) * cell, my + (self.player_y / CELL_SIZE) * cell, 4.0, YELLOW);
        draw_circle_lines(mx + (self.player_x / CELL_SIZE) * cell, my + (self.player_y / CELL_SIZE) * cell, 4.0, 1.0, BLACK);
        
        // Other players on minimap (visible and distinctive)
        self.multiplayer.draw_other_players_on_minimap(mx, my, cell, self.level as i32);
        
        // Player direction indicator - accurate angle representation
        let px = mx + (self.player_x / CELL_SIZE) * cell;
        let py = my + (self.player_y / CELL_SIZE) * cell;
        let dir_length = 12.0;
        let dir_x = px + self.player_angle.cos() * dir_length;
        let dir_y = py + self.player_angle.sin() * dir_length;
        draw_line(px, py, dir_x, dir_y, 3.0, Color::new(1.0, 1.0, 0.0, 1.0)); // Bright yellow
        
        // Player coordinates display
        let coord_text = format!("X:{:.1} Y:{:.1}", self.player_x / CELL_SIZE, self.player_y / CELL_SIZE);
        draw_text(&coord_text, mx, my + size + 15.0, 14.0, WHITE);
    }
    
    
    fn draw_stone_wall_column(&self, x: f32, y: f32, width: f32, height: f32, column_index: usize) {
        // Modern minimalist stone - clean geometric design
        let base_color = Color::from_rgba(245, 245, 240, 255); // Warm white stone
        draw_rectangle(x, y, width, height, base_color);
        
        // Subtle geometric patterns
        let panel_height = 25.0;
        let mut panel_y = y;
        let mut panel_row = 0;
        
        while panel_y < y + height {
            let remaining_height = (y + height - panel_y).min(panel_height);
            
            // Clean separation lines
            draw_rectangle(x, panel_y, width, 1.0, Color::from_rgba(220, 220, 215, 255));
            
            // Subtle depth variation
            if column_index % 3 == 0 {
                draw_rectangle(x + 2.0, panel_y + 2.0, width - 4.0, remaining_height - 3.0, Color::from_rgba(235, 235, 230, 255));
            }
            
            // Modern accent lines
            if panel_row % 2 == 0 {
                draw_rectangle(x, panel_y + remaining_height * 0.7, width, 1.0, Color::from_rgba(200, 200, 195, 255));
            }
            
            panel_y += panel_height;
            panel_row += 1;
        }
    }
    
    fn draw_metal_wall_column(&self, x: f32, y: f32, width: f32, height: f32, column_index: usize) {
        // Modern tech aesthetic - sleek and professional
        let base_color = Color::from_rgba(45, 55, 70, 255); // Deep blue-gray
        draw_rectangle(x, y, width, height, base_color);
        
        // Clean tech panels
        let panel_height = 35.0;
        let mut panel_y = y;
        
        while panel_y < y + height {
            let remaining_height = (y + height - panel_y).min(panel_height);
            
            // Sleek panel borders
            draw_rectangle(x, panel_y, width, 1.0, Color::from_rgba(120, 140, 180, 255));
            draw_rectangle(x, panel_y + remaining_height - 1.0, width, 1.0, Color::from_rgba(120, 140, 180, 255));
            
            // Modern accent strips
            if column_index % 3 == 0 {
                draw_rectangle(x + 2.0, panel_y + 5.0, width - 4.0, 2.0, Color::from_rgba(100, 150, 255, 180));
            }
            
            // Subtle tech details
            if column_index % 5 == 0 {
                draw_rectangle(x + width * 0.1, panel_y + remaining_height * 0.3, 2.0, 2.0, Color::from_rgba(150, 200, 255, 200));
                draw_rectangle(x + width * 0.9 - 2.0, panel_y + remaining_height * 0.7, 2.0, 2.0, Color::from_rgba(150, 200, 255, 200));
            }
            
            panel_y += panel_height;
        }
    }
    
    fn draw_crystal_wall_column(&self, x: f32, y: f32, width: f32, height: f32, column_index: usize) {
        // Elegant modern crystal - sophisticated and clean
        let base_color = Color::from_rgba(240, 248, 255, 255); // Alice blue - very clean
        draw_rectangle(x, y, width, height, base_color);
        
        // Geometric crystal patterns
        let crystal_height = 28.0;
        let mut crystal_y = y;
        
        while crystal_y < y + height {
            let remaining_height = (y + height - crystal_y).min(crystal_height);
            
            // Modern geometric facets
            if column_index % 2 == 0 {
                draw_rectangle(x + 1.0, crystal_y + 1.0, width * 0.25, remaining_height - 2.0, Color::from_rgba(220, 235, 255, 255));
                draw_rectangle(x + width * 0.75, crystal_y + 1.0, width * 0.24, remaining_height - 2.0, Color::from_rgba(220, 235, 255, 255));
            }
            
            // Subtle energy lines (no harsh pulsing)
            let subtle_glow = (get_time() * 1.0 + column_index as f64 * 0.2).sin() * 0.1 + 0.9;
            let energy_color = Color::new(0.4, 0.7, 1.0, subtle_glow as f32 * 0.3);
            draw_rectangle(x + width * 0.5 - 0.5, crystal_y, 1.0, remaining_height, energy_color);
            
            // Clean separation lines
            draw_rectangle(x, crystal_y, width, 1.0, Color::from_rgba(200, 220, 240, 255));
            
            crystal_y += crystal_height;
        }
    }

    fn draw_textured_wall_column(&self, x: f32, y: f32, width: f32, height: f32, level: u32, column_index: usize) {
        match level {
            1 => self.draw_stone_wall_column(x, y, width, height, column_index),
            2 => self.draw_metal_wall_column(x, y, width, height, column_index),
            3 => self.draw_crystal_wall_column(x, y, width, height, column_index),
            _ => draw_rectangle(x, y, width, height, GRAY),
        }
    }

    fn generate_dense_maze_pattern(&mut self) {
        // Fill entire maze with walls first
        for y in 0..self.maze_height {
            for x in 0..self.maze_width {
                self.maze[y][x] = true;
            }
        }
        
        // Keep borders as walls but create internal maze structure
        match self.level {
            1 => self.generate_level1_pattern(),
            2 => self.generate_level2_pattern(), 
            3 => self.generate_level3_pattern(),
            _ => self.generate_level1_pattern(),
        };
        
        // Ensure player starting area is clear
        for dy in -1..=1 {
            for dx in -1..=1 {
                let px = 3 + dx;
                let py = 3 + dy;
                if px >= 0 && px < self.maze_width as i32 && py >= 0 && py < self.maze_height as i32 {
                    self.maze[py as usize][px as usize] = false;
                }
            }
        }
        
        // Ensure exit area is accessible
        for dy in -1..=1 {
            for dx in -1..=1 {
                let ex = (self.exit_x / CELL_SIZE) as i32 + dx;
                let ey = (self.exit_y / CELL_SIZE) as i32 + dy;
                if ex >= 0 && ex < self.maze_width as i32 && ey >= 0 && ey < self.maze_height as i32 {
                    self.maze[ey as usize][ex as usize] = false;
                }
            }
        }
        
        // Create guaranteed path from start to exit
        self.create_solution_path();
    }
    
    fn setup_networking(&mut self) {
        self.multiplayer.setup_networking(&self.server_addr, &self.username);
    }
    
    
    
    
    fn shoot_bullet(&mut self) {
        if self.ammo > 0 {
            self.ammo -= 1;
            self.gun_fire_t = 0.2;
            self.gun_recoil = 1.0;
            
            // Create bullet
            let bullet_speed = 800.0;
            let dx = self.player_angle.cos() * bullet_speed;
            let dy = self.player_angle.sin() * bullet_speed;
            
            // Add to local bullets
            self.multiplayer.bullets.push(Bullet {
                x: self.player_x,
                y: self.player_y,
                dx,
                dy,
                owner: self.username.clone(),
                created: Instant::now(),
            });
            
            // Send to server
            self.multiplayer.send_shoot(self.player_x, self.player_y, dx, dy);
        }
    }
    
    fn recharge_ammo(&mut self) {
        const MAX_AMMO: i32 = 30;
        const RECHARGE_AMOUNT: i32 = 30;
        
        if self.ammo < MAX_AMMO {
            self.ammo = (self.ammo + RECHARGE_AMOUNT).min(MAX_AMMO);
            
            // Add visual feedback
            self.crosshair_pulse = 1.0;
        }
    }
    
    
    
    fn draw_score_display(&self) {
        let (kills, deaths) = self.multiplayer.get_score();
        let kd_ratio = if deaths > 0 { kills as f32 / deaths as f32 } else { kills as f32 };
        
        // Score panel background
        let panel_width = 200.0;
        let panel_height = 80.0;
        let panel_x = screen_width() - panel_width - 10.0;
        let panel_y = screen_height() - panel_height - 10.0;
        
        draw_rectangle(panel_x, panel_y, panel_width, panel_height, Color::from_rgba(0, 0, 0, 150));
        draw_rectangle_lines(panel_x, panel_y, panel_width, panel_height, 2.0, WHITE);
        
        // Score title
        draw_text("SCOREBOARD", panel_x + 10.0, panel_y + 20.0, 16.0, YELLOW);
        
        // Stats
        let kills_text = format!("Kills: {}", kills);
        let deaths_text = format!("Deaths: {}", deaths);
        let kd_text = format!("K/D: {:.2}", kd_ratio);
        
        draw_text(&kills_text, panel_x + 10.0, panel_y + 40.0, 14.0, GREEN);
        draw_text(&deaths_text, panel_x + 10.0, panel_y + 55.0, 14.0, RED);
        draw_text(&kd_text, panel_x + 10.0, panel_y + 70.0, 14.0, WHITE);
    }
    
    fn generate_level1_pattern(&mut self) {
        // Level 1: Harder - more walls and complex maze (65% wall density)
        for y in 1..(self.maze_height - 1) {
            for x in 1..(self.maze_width - 1) {
                // Create denser grid pattern with fewer openings
                if (x % 4 == 1) || (y % 4 == 1) {
                    self.maze[y][x] = false;
                }
                
                // Higher wall density - more challenging to navigate
                let noise = ((x * 7 + y * 11) % 100) as f32 / 100.0;
                if noise < 0.65 {
                    self.maze[y][x] = true;
                }
            }
        }
        
        // Create fewer connecting corridors - more challenging navigation
        for y in 3..(self.maze_height - 3) {
            for x in 3..(self.maze_width - 3) {
                let pattern = (x * 3 + y * 5) % 8;
                if pattern < 2 {
                    // Create narrow openings and limited connections
                    self.maze[y][x] = false;
                    
                    // Add limited connecting paths
                    if pattern == 0 {
                        if y > 3 { self.maze[y - 1][x] = false; }
                    } else if pattern == 1 {
                        if y < self.maze_height - 3 { self.maze[y + 1][x] = false; }
                        if x < self.maze_width - 3 { self.maze[y][x + 1] = false; }
                    }
                }
            }
        }
        
        // Add more walls and dead ends for increased difficulty
        for y in 2..(self.maze_height - 2) {
            for x in 2..(self.maze_width - 2) {
                // Create more dead ends and complex passages
                let complexity = ((x * 13 + y * 17) % 100) as f32 / 100.0;
                if complexity < 0.3 {
                    // Add extra walls to create dead ends
                    self.maze[y][x] = true;
                    if x > 2 && complexity < 0.15 { self.maze[y][x - 1] = true; }
                    if y > 2 && complexity < 0.15 { self.maze[y - 1][x] = true; }
                }
            }
        }
    }
    
    fn generate_level2_pattern(&mut self) {
        // Level 2: Moderate - balanced challenge with some tight passages (55% wall density)
        for y in 1..(self.maze_height - 1) {
            for x in 1..(self.maze_width - 1) {
                // Create moderate corridor structure
                if (x % 4 == 1) || (y % 4 == 1) {
                    self.maze[y][x] = false;
                }
                
                // Moderate wall density - balanced challenge
                let noise = ((x * 13 + y * 17) % 100) as f32 / 100.0;
                if noise < 0.55 {
                    self.maze[y][x] = true;
                }
            }
        }
        
        // Create moderate spiral corridors
        let center_x = self.maze_width / 2;
        let center_y = self.maze_height / 2;
        for y in 3..(self.maze_height - 3) {
            for x in 3..(self.maze_width - 3) {
                let dx = x as i32 - center_x as i32;
                let dy = y as i32 - center_y as i32;
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                let angle = (dy as f32).atan2(dx as f32);
                
                // Create moderate spiral corridors
                let spiral = (dist * 0.8 + angle * 2.0).sin();
                if spiral > 0.4 && dist < 12.0 && dist > 3.0 {
                    self.maze[y][x] = false;
                }
            }
        }
        
        // Create moderate strategic openings
        for y in 3..(self.maze_height - 3) {
            for x in 3..(self.maze_width - 3) {
                let pattern = (x * 11 + y * 13) % 12;
                if pattern < 4 {
                    // Create moderate L-shaped openings
                    self.maze[y][x] = false;
                    if pattern < 2 {
                        self.maze[y + 1][x] = false;
                        self.maze[y][x + 1] = false;
                    }
                }
            }
        }
        
        // Add moderate grid pattern
        for y in 2..(self.maze_height - 2) {
            for x in 2..(self.maze_width - 2) {
                if (x % 6 == 2 && y % 3 == 1) || (y % 6 == 2 && x % 3 == 1) {
                    self.maze[y][x] = false;
                }
            }
        }
    }
    
    fn generate_level3_pattern(&mut self) {
        // Level 3: Maximum density - extremely challenging (85% wall density)
        for y in 1..(self.maze_height - 1) {
            for x in 1..(self.maze_width - 1) {
                // Start with almost all walls
                let noise = ((x * 11 + y * 7) % 100) as f32 / 100.0;
                if noise < 0.85 {
                    self.maze[y][x] = true;
                }
            }
        }
        
        // Create extremely sparse corridor network - only essential paths
        for y in 3..(self.maze_height - 3) {
            for x in 3..(self.maze_width - 3) {
                // Very sparse grid - only every 10th cell gets a corridor
                if (x % 10 == 5) && (y % 10 == 5) {
                    self.maze[y][x] = false;
                    // Create tiny cross pattern
                    self.maze[y - 1][x] = false;
                    self.maze[y + 1][x] = false;
                    self.maze[y][x - 1] = false;
                    self.maze[y][x + 1] = false;
                }
            }
        }
        
        // Add minimal winding paths - very few openings
        for y in 2..(self.maze_height - 2) {
            for x in 2..(self.maze_width - 2) {
                let path_pattern = (x * 17 + y * 19) % 25;
                if path_pattern < 2 {
                    // Create single-cell openings with rare connections
                    self.maze[y][x] = false;
                    
                    // Very rarely connect to adjacent cells
                    if path_pattern == 0 {
                        let direction = (x + y) % 4;
                        match direction {
                            0 => if y > 1 { self.maze[y - 1][x] = false; },
                            1 => if x < self.maze_width - 2 { self.maze[y][x + 1] = false; },
                            2 => if y < self.maze_height - 2 { self.maze[y + 1][x] = false; },
                            3 => if x > 1 { self.maze[y][x - 1] = false; },
                            _ => {}
                        }
                    }
                }
            }
        }
        
        // Create extremely narrow diagonal passages - very rare
        for y in 4..(self.maze_height - 4) {
            for x in 4..(self.maze_width - 4) {
                let diagonal_pattern = (x * 23 + y * 29) % 30;
                if diagonal_pattern < 1 {
                    // Create tiny diagonal corridor
                    self.maze[y][x] = false;
                    self.maze[y + 1][x + 1] = false;
                }
            }
        }
        
        // Add fractal-like micro-patterns - extremely sparse
        for y in 6..(self.maze_height - 6) {
            for x in 6..(self.maze_width - 6) {
                let fractal_pattern = (x * 31 + y * 37) % 40;
                if fractal_pattern < 1 {
                    // Create tiny fractal opening
                    self.maze[y][x] = false;
                    
                    // Micro-branches - very small
                    let micro_branch = (x + y) % 3;
                    if micro_branch == 0 && y > 1 {
                        self.maze[y - 1][x] = false;
                    }
                }
            }
        }
        
        // Final pass - create maximum dead ends
        for y in 2..(self.maze_height - 2) {
            for x in 2..(self.maze_width - 2) {
                if !self.maze[y][x] {
                    let neighbors = self.count_wall_neighbors(x, y);
                    // Create maximum dead ends
                    if neighbors >= 1 {
                        let noise = ((x * 19 + y * 23) % 100) as f32 / 100.0;
                        if noise < 0.7 {
                            self.maze[y][x] = true;
                        }
                    }
                }
            }
        }
    }
    
    fn count_wall_neighbors(&self, x: usize, y: usize) -> usize {
        let mut count = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 { continue; }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && nx < self.maze_width as i32 && ny >= 0 && ny < self.maze_height as i32 {
                    if self.maze[ny as usize][nx as usize] {
                        count += 1;
                    }
                }
            }
        }
        count
    }
    
    fn create_solution_path(&mut self) {
        // Use A* pathfinding to ensure there's always a solution
        let start_x = 3;
        let start_y = 3;
        let end_x = (self.exit_x / CELL_SIZE) as usize;
        let end_y = (self.exit_y / CELL_SIZE) as usize;
        
        // Simple path creation - carve direct path with some randomness
        let mut current_x = start_x;
        let mut current_y = start_y;
        
        while current_x != end_x || current_y != end_y {
            self.maze[current_y][current_x] = false;
            
            // Move towards target with some randomness
            let dx = if current_x < end_x { 1 } else if current_x > end_x { -1 } else { 0 };
            let dy = if current_y < end_y { 1 } else if current_y > end_y { -1 } else { 0 };
            
            // Add randomness to path
            let noise = ((current_x * 7 + current_y * 11) % 100) as f32 / 100.0;
            if noise < 0.3 && dx != 0 {
                current_x = (current_x as i32 + dx).max(1).min(self.maze_width as i32 - 2) as usize;
            } else if noise < 0.6 && dy != 0 {
                current_y = (current_y as i32 + dy).max(1).min(self.maze_height as i32 - 2) as usize;
            } else {
                // Move in primary direction
                if (current_x as i32 - end_x as i32).abs() > (current_y as i32 - end_y as i32).abs() {
                    current_x = (current_x as i32 + dx).max(1).min(self.maze_width as i32 - 2) as usize;
                } else {
                    current_y = (current_y as i32 + dy).max(1).min(self.maze_height as i32 - 2) as usize;
                }
            }
        }
        
        // Ensure final path to exit
        self.maze[end_y][end_x] = false;
    }

    fn draw_3d_exit_door(&self) {
        let sw = screen_width();
        let sh = screen_height();
        
        // Calculate distance and angle to exit
        let dx = self.exit_x - self.player_x;
        let dy = self.exit_y - self.player_y;
        let dist = (dx * dx + dy * dy).sqrt();
        
        // Only draw if exit is very close (within 2-3 cells distance)
        if dist > 120.0 {
            return;
        }
        
        let angle_to_exit = dy.atan2(dx);
        let mut rel_angle = angle_to_exit - self.player_angle;
        
        // Normalize angle
        while rel_angle > std::f32::consts::PI {
            rel_angle -= 2.0 * std::f32::consts::PI;
        }
        while rel_angle < -std::f32::consts::PI {
            rel_angle += 2.0 * std::f32::consts::PI;
        }
        
        // Only draw if exit is in front of player (within FOV)
        if rel_angle.abs() > FOV / 2.0 {
            return;
        }
        
        // Calculate screen position
        let screen_x = sw / 2.0 + (rel_angle / FOV) * sw;
        
        // Calculate door size based on distance
        let base_size = 200.0;
        let door_size = base_size / (dist / CELL_SIZE + 1.0);
        let door_height = door_size * 1.5;
        
        if door_size < 10.0 {
            return; // Too far to see clearly
        }
        
        let door_x = screen_x - door_size / 2.0;
        let door_y = sh / 2.0 - door_height / 2.0;
        
        // Pulsing effect
        let pulse = (get_time() * 4.0).sin() * 0.3 + 0.7;
        let proximity_factor = (1.0 - (dist / 400.0).min(1.0)).max(0.0);
        
        // Draw themed exit door based on level
        match self.level {
            1 => {
                // Golden Arch Door
                let gold_color = Color::new(1.0, 0.84, 0.0, pulse as f32 * proximity_factor + 0.3);
                let arch_color = Color::new(0.8, 0.65, 0.0, 0.8);
                
                // Door frame (arch shape)
                draw_rectangle(door_x - 5.0, door_y, door_size + 10.0, door_height, arch_color);
                draw_rectangle(door_x, door_y + 10.0, door_size, door_height - 10.0, gold_color);
                
                // Arch top
                draw_circle(screen_x, door_y + 10.0, door_size / 2.0 + 5.0, arch_color);
                draw_circle(screen_x, door_y + 10.0, door_size / 2.0, gold_color);
                
                // Door details
                draw_rectangle(door_x + door_size * 0.1, door_y + door_height * 0.2, 
                              door_size * 0.8, door_height * 0.6, Color::new(0.6, 0.4, 0.2, 0.9));
                
                // Door handle
                draw_circle(door_x + door_size * 0.8, door_y + door_height * 0.5, 3.0, GOLD);
            },
            
            2 => {
                // Cyberpunk Portal
                let portal_color = Color::new(0.0, 1.0, 1.0, pulse as f32 * proximity_factor + 0.4);
                let frame_color = Color::new(0.5, 0.0, 1.0, 0.8);
                
                // Portal frame
                draw_rectangle_lines(door_x - 8.0, door_y - 8.0, door_size + 16.0, door_height + 16.0, 4.0, frame_color);
                
                // Swirling portal effect
                let time = get_time() as f32;
                for i in 0..8 {
                    let angle = time * 2.0 + (i as f32 * std::f32::consts::PI / 4.0);
                    let radius = door_size * 0.3 * (1.0 + (time + i as f32).sin() * 0.2);
                    let px = screen_x + angle.cos() * radius * 0.5;
                    let py = door_y + door_height * 0.5 + angle.sin() * radius * 0.3;
                    draw_circle(px, py, 4.0, portal_color);
                }
                
                // Portal center
                draw_circle(screen_x, door_y + door_height * 0.5, door_size * 0.2, portal_color);
            },
            
            3 => {
                // Crystal Gate
                let crystal_color = Color::new(1.0, 0.5, 1.0, pulse as f32 * proximity_factor + 0.4);
                let gate_color = Color::new(0.8, 0.8, 1.0, 0.7);
                
                // Crystal gate frame
                let points = [
                    Vec2::new(screen_x, door_y),
                    Vec2::new(door_x + door_size, door_y + door_height * 0.3),
                    Vec2::new(door_x + door_size, door_y + door_height * 0.7),
                    Vec2::new(screen_x, door_y + door_height),
                    Vec2::new(door_x, door_y + door_height * 0.7),
                    Vec2::new(door_x, door_y + door_height * 0.3),
                ];
                
                // Draw crystal shape
                for i in 0..points.len() {
                    let next = (i + 1) % points.len();
                    draw_line(points[i].x, points[i].y, points[next].x, points[next].y, 3.0, gate_color);
                }
                
                // Crystal sparkles
                let time = get_time() as f32;
                for i in 0..12 {
                    let sparkle_time = time * 3.0 + i as f32;
                    if sparkle_time.sin() > 0.5 {
                        let sx = door_x + (i as f32 / 12.0) * door_size;
                        let sy = door_y + ((i * 7) % 13) as f32 / 13.0 * door_height;
                        draw_circle(sx, sy, 2.0, crystal_color);
                    }
                }
                
                // Central crystal
                draw_circle(screen_x, door_y + door_height * 0.5, door_size * 0.15, crystal_color);
            },
            
            _ => {
                // Default simple door
                let door_color = Color::new(0.8, 0.4, 0.2, 0.8);
                draw_rectangle(door_x, door_y, door_size, door_height, door_color);
                draw_rectangle_lines(door_x, door_y, door_size, door_height, 2.0, Color::new(0.6, 0.3, 0.1, 1.0));
                draw_circle(door_x + door_size * 0.8, door_y + door_height * 0.5, 3.0, YELLOW);
            }
        }
    }
}

// ----------------- boilerplate: input & window -----------------
fn window_conf() -> Conf {
    Conf {
        window_title: "Multiplayer FPS".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: false,
        fullscreen: false,
        sample_count: 1,
        window_resizable: true,
        icon: None,
        platform: Default::default(),
    }
}

fn get_user_input() -> Result<(String, String), Box<dyn std::error::Error>> {
    println!("=== Multiplayer FPS Client ===");
    print!("Enter IP Address (e.g. 127.0.0.1:34254): ");
    io::stdout().flush().unwrap();
    let mut ip = String::new();
    io::stdin().read_line(&mut ip)?;
    let server_addr = if ip.trim().is_empty() {
        "127.0.0.1:34254".to_string()
    } else {
        ip.trim().to_string()
    };

    print!("Enter Name: ");
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let username = name.trim().to_string();

    println!("Starting...");
    println!("Connecting to server: {}", server_addr);
    println!("Username: {}", username);

    Ok((username, server_addr))
}

#[macroquad::main(window_conf)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (username, server_addr) = get_user_input()?;


    let mut gs = GameState::new(username, server_addr);
    
    // Setup networking
    gs.setup_networking();
    
    // Load assets
    if let Err(e) = gs.assets.load_weapon_sprites().await {
        println!("Warning: Failed to load weapon sprites: {}", e);
    }
    if let Err(e) = gs.assets.load_ui_sprites().await {
        println!("Warning: Failed to load UI sprites: {}", e);
    }

    // frame limiter
    let target_frame = Duration::from_secs_f64(1.0 / TARGET_FPS as f64);

    loop {
        let start = Instant::now();

        // Check for ESC key to exit (especially important for victory screen)
        if is_key_pressed(KeyCode::Escape) {
            break Ok(());
        }

        gs.update();
        gs.draw();

        let elapsed = start.elapsed();
        if elapsed < target_frame {
            std::thread::sleep(target_frame - elapsed);
        }

        next_frame().await;
    }
}
