use macroquad::prelude::*;
use std::net::UdpSocket;
use std::time::Instant;
use std::io::{self, Write};
use std::f32::consts::PI;

mod themes;
use themes::{LevelTheme, ThemeConfig};

const MAZE_WIDTH: usize = 16;
const MAZE_HEIGHT: usize = 16;
const CELL_SIZE: f32 = 64.0;
const FOV: f32 = PI / 3.0; // 60 degrees field of view
const RENDER_DISTANCE: f32 = 1000.0;

#[derive(Clone, Copy)]
struct Enemy {
    x: f32,
    y: f32,
    angle: f32,
    health: i32,
    patrol_target_x: f32,
    patrol_target_y: f32,
    last_seen_player: Instant,
    state: EnemyState,
}

#[derive(Clone, Copy)]
enum EnemyState {
    Patrolling,
    Chasing,
    Attacking,
}

impl Enemy {
    fn new(x: f32, y: f32, angle: f32) -> Self {
        Self {
            x,
            y,
            angle,
            health: 50,
            patrol_target_x: x,
            patrol_target_y: y,
            last_seen_player: Instant::now(),
            state: EnemyState::Patrolling,
        }
    }
}

struct GameState {
    player_x: f32,
    player_y: f32,
    player_angle: f32,
    maze: [[bool; MAZE_WIDTH]; MAZE_HEIGHT],
    level: usize,
    score: i32,
    exit_x: f32,
    exit_y: f32,
    server_addr: String,
    username: String,
    mouse_sensitivity: f32,
    last_mouse_x: f32,
    frame_start: Instant,
    frame_times: Vec<f32>,
    health: i32,
    ammo: i32,
    game_won: bool,
    last_frame_time: Instant,
    fps_counter: f32,
    crosshair_pulse: f32,
    wall_hit_flash: f32,
    enemies: Vec<Enemy>,
    last_enemy_attack: Instant,
    current_theme: LevelTheme,
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
            player_x: 3.5 * CELL_SIZE,
            player_y: 3.5 * CELL_SIZE,
            player_angle: 0.0,
            maze,
            exit_x: 13.5 * CELL_SIZE,
            exit_y: 13.5 * CELL_SIZE,
            server_addr,
            username,
            mouse_sensitivity: 0.003,
            last_mouse_x: 0.0,
            frame_start: Instant::now(),
            frame_times: Vec::with_capacity(60),
            health: 100,
            ammo: 30,
            level: 1,
            score: 0,
            game_won: false,
            last_frame_time: Instant::now(),
            fps_counter: 60.0,
            crosshair_pulse: 0.0,
            wall_hit_flash: 0.0,
            enemies: Vec::new(),
            last_enemy_attack: Instant::now(),
            current_theme: LevelTheme::CandyMaze,
        }
    }

    fn update(&mut self) {
        // Calculate FPS with smoothing
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time).as_secs_f32();
        let current_fps = 1.0 / delta.max(0.001);
        
        // Add to frame times buffer for smooth FPS calculation
        self.frame_times.push(current_fps);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }
        
        // Calculate average FPS
        self.fps_counter = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        self.last_frame_time = now;

        // Update animations
        self.crosshair_pulse += delta * 3.0;
        self.wall_hit_flash = (self.wall_hit_flash - delta * 2.0).max(0.0);

        // Mouse look (professional FPS controls)
        let (mouse_x, _) = mouse_position();
        if self.last_mouse_x != 0.0 {
            let mouse_delta = mouse_x - self.last_mouse_x;
            self.player_angle += mouse_delta * self.mouse_sensitivity;
        }
        self.last_mouse_x = mouse_x;

        let move_speed = 200.0 * delta; // Increased for better feel
        let strafe_speed = 180.0 * delta;

        // Professional FPS movement (WASD + mouse)
        let mut new_x = self.player_x;
        let mut new_y = self.player_y;

        // Handle input
        if is_key_down(KeyCode::W) {
            new_x += self.player_angle.cos() * move_speed;
            new_y += self.player_angle.sin() * move_speed;
        }
        if is_key_down(KeyCode::S) {
            new_x -= self.player_angle.cos() * move_speed;
            new_y -= self.player_angle.sin() * move_speed;
        }
        if is_key_down(KeyCode::A) {
            new_x += (self.player_angle - PI/2.0).cos() * strafe_speed;
            new_y += (self.player_angle - PI/2.0).sin() * strafe_speed;
        }
        if is_key_down(KeyCode::D) {
            new_x += (self.player_angle + PI/2.0).cos() * strafe_speed;
            new_y += (self.player_angle + PI/2.0).sin() * strafe_speed;
        }
        
        // Debug: Cycle through themes with T key
        if is_key_pressed(KeyCode::T) {
            self.current_theme = match self.current_theme {
                LevelTheme::CandyMaze => LevelTheme::Cyberpunk,
                LevelTheme::Cyberpunk => LevelTheme::MoroccanBazaar,
                LevelTheme::MoroccanBazaar => LevelTheme::CandyMaze,
            };
        }

        // Arrow keys for keyboard-only players
        if is_key_down(KeyCode::Left) {
            self.player_angle -= 2.0 * delta;
        }
        if is_key_down(KeyCode::Right) {
            self.player_angle += 2.0 * delta;
        }
        if is_key_down(KeyCode::Up) {
            new_x += self.player_angle.cos() * move_speed;
            new_y += self.player_angle.sin() * move_speed;
        }
        if is_key_down(KeyCode::Down) {
            new_x -= self.player_angle.cos() * move_speed;
            new_y -= self.player_angle.sin() * move_speed;
        }

        // Enhanced collision detection with wall hit feedback
        if !self.is_wall(new_x, new_y) {
            self.player_x = new_x;
            self.player_y = new_y;
        } else {
            // Wall hit effect
            self.wall_hit_flash = 0.3;
        }

        // Check if player reached the exit
        let distance_to_exit = ((self.player_x - self.exit_x).powi(2) + (self.player_y - self.exit_y).powi(2)).sqrt();
        if distance_to_exit < 40.0 {
            self.advance_level();
        }

        // Shooting mechanics with enhanced feedback
        if is_key_pressed(KeyCode::Space) && self.ammo > 0 {
            self.shoot();
            self.crosshair_pulse = 0.0; // Reset crosshair animation
        }
        
        // Update AI enemies
        self.update_enemies(delta);
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

        // Update theme based on level
        self.current_theme = LevelTheme::from_level(self.level);

        // Generate new maze based on level
        self.generate_maze_for_level(self.level as i32);
        
        // Spawn enemies for this level
        self.spawn_enemies();
        
        // Reset health and ammo for new level
        self.health = 100;
        self.ammo = 30;
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
        if self.ammo > 0 {
            self.ammo -= 1;
            // Add muzzle flash effect
            self.wall_hit_flash = 0.2;
            
            // Check if we hit any enemies
            let hit_enemy = self.check_enemy_hit();
            if hit_enemy {
                self.score += 50; // More points for hitting enemies
            } else {
                self.score += 10; // Base shooting points
            }
        }
    }
    
    fn check_enemy_hit(&mut self) -> bool {
        let ray_cos = self.player_angle.cos();
        let ray_sin = self.player_angle.sin();
        let player_x = self.player_x;
        let player_y = self.player_y;
        let player_angle = self.player_angle;
        
        for enemy in &mut self.enemies {
            if enemy.health <= 0 { continue; }
            
            // Calculate distance to enemy
            let dx = enemy.x - player_x;
            let dy = enemy.y - player_y;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance > 300.0 { continue; } // Max shooting range
            
            // Check if enemy is in line of sight
            let angle_to_enemy = dy.atan2(dx);
            let angle_diff = (angle_to_enemy - player_angle).abs();
            
            if angle_diff < 0.1 { // Small angle tolerance for hitting
                // Check for walls between player and enemy
                let steps = (distance / 5.0) as i32;
                let mut blocked = false;
                
                for i in 1..steps {
                    let check_x = player_x + ray_cos * (i as f32 * 5.0);
                    let check_y = player_y + ray_sin * (i as f32 * 5.0);
                    let grid_x = (check_x / CELL_SIZE) as usize;
                    let grid_y = (check_y / CELL_SIZE) as usize;
                    
                    if grid_x >= MAZE_WIDTH || grid_y >= MAZE_HEIGHT || self.maze[grid_y][grid_x] {
                        blocked = true;
                        break;
                    }
                }
                
                if !blocked {
                    enemy.health -= 25; // Damage per hit
                    return true;
                }
            }
        }
        false
    }
    
    fn spawn_enemies(&mut self) {
        self.enemies.clear();
        let num_enemies = self.level + 1; // More enemies each level
        
        for i in 0..num_enemies {
            // Find valid spawn positions (not in walls, not near player)
            let mut attempts = 0;
            while attempts < 50 {
                let x = (4 + (i * 3) % 8) as f32 * CELL_SIZE + CELL_SIZE / 2.0;
                let y = (4 + (i * 2) % 8) as f32 * CELL_SIZE + CELL_SIZE / 2.0;
                
                if !self.is_wall(x, y) {
                    let distance_to_player = ((x - self.player_x).powi(2) + (y - self.player_y).powi(2)).sqrt();
                    if distance_to_player > 200.0 { // Not too close to player
                        self.enemies.push(Enemy::new(x, y, (i as f32 * PI / 2.0) % (2.0 * PI)));
                        break;
                    }
                }
                attempts += 1;
            }
        }
    }
    
    fn update_enemies(&mut self, delta: f32) {
        let player_pos = (self.player_x, self.player_y);
        let maze = self.maze; // Copy maze for borrowing
        
        // Collect enemy updates to avoid borrowing issues
        let mut enemy_updates = Vec::new();
        let mut attack_occurred = false;
        
        for (i, enemy) in self.enemies.iter().enumerate() {
            if enemy.health <= 0 { continue; }
            
            let distance_to_player = ((enemy.x - player_pos.0).powi(2) + (enemy.y - player_pos.1).powi(2)).sqrt();
            
            let mut new_enemy = *enemy;
            
            // AI State Machine
            match enemy.state {
                EnemyState::Patrolling => {
                    // Simple patrol movement
                    let move_speed = 50.0 * delta;
                    let new_x = enemy.x + enemy.angle.cos() * move_speed;
                    let new_y = enemy.y + enemy.angle.sin() * move_speed;
                    
                    // Check walls manually
                    let grid_x = (new_x / CELL_SIZE) as usize;
                    let grid_y = (new_y / CELL_SIZE) as usize;
                    let is_wall = grid_x >= MAZE_WIDTH || grid_y >= MAZE_HEIGHT || maze[grid_y][grid_x];
                    
                    if !is_wall {
                        new_enemy.x = new_x;
                        new_enemy.y = new_y;
                    } else {
                        new_enemy.angle += PI / 2.0; // Turn 90 degrees
                    }
                    
                    // Switch to chasing if player is close
                    if distance_to_player < 150.0 {
                        new_enemy.state = EnemyState::Chasing;
                        new_enemy.last_seen_player = Instant::now();
                    }
                }
                EnemyState::Chasing => {
                    // Move towards player
                    let angle_to_player = (player_pos.1 - enemy.y).atan2(player_pos.0 - enemy.x);
                    new_enemy.angle = angle_to_player;
                    
                    let chase_speed = 80.0 * delta;
                    let new_x = enemy.x + new_enemy.angle.cos() * chase_speed;
                    let new_y = enemy.y + new_enemy.angle.sin() * chase_speed;
                    
                    // Check walls manually
                    let grid_x = (new_x / CELL_SIZE) as usize;
                    let grid_y = (new_y / CELL_SIZE) as usize;
                    let is_wall = grid_x >= MAZE_WIDTH || grid_y >= MAZE_HEIGHT || maze[grid_y][grid_x];
                    
                    if !is_wall {
                        new_enemy.x = new_x;
                        new_enemy.y = new_y;
                    }
                    
                    // Attack if very close
                    if distance_to_player < 50.0 {
                        new_enemy.state = EnemyState::Attacking;
                    }
                    
                    // Return to patrol if player is far
                    if distance_to_player > 200.0 {
                        new_enemy.state = EnemyState::Patrolling;
                    }
                }
                EnemyState::Attacking => {
                    // Attack player
                    if self.last_enemy_attack.elapsed().as_secs_f32() > 1.0 {
                        attack_occurred = true;
                    }
                    
                    // Return to chasing if not close enough
                    if distance_to_player > 60.0 {
                        new_enemy.state = EnemyState::Chasing;
                    }
                }
            }
            
            enemy_updates.push((i, new_enemy));
        }
        
        // Apply updates
        for (i, updated_enemy) in enemy_updates {
            if i < self.enemies.len() {
                self.enemies[i] = updated_enemy;
            }
        }
        
        // Handle attack
        if attack_occurred {
            self.health -= 10;
            self.last_enemy_attack = Instant::now();
            self.wall_hit_flash = 0.5; // Red flash when hit
        }
        
        // Remove dead enemies
        self.enemies.retain(|e| e.health > 0);
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
        self.draw_enhanced_hud();
    }

    fn draw_3d_view(&self) {
        let screen_width = screen_width();
        let screen_height = screen_height();
        let num_rays = 320;
        let theme = self.current_theme.get_config();
        
        // Draw themed floor and ceiling
        draw_rectangle(0.0, 0.0, screen_width, screen_height / 2.0, theme.ceiling_color);
        draw_rectangle(0.0, screen_height / 2.0, screen_width, screen_height / 2.0, theme.floor_color);
        
        // Add theme-specific atmospheric effects
        self.current_theme.draw_atmospheric_effects(&theme, self.crosshair_pulse);
        
        for i in 0..num_rays {
            let ray_angle = self.player_angle - FOV / 2.0 + (i as f32 / num_rays as f32) * FOV;
            
            // Enhanced raycast with better precision
            let mut distance = 0.0;
            let ray_cos = ray_angle.cos();
            let ray_sin = ray_angle.sin();
            
            while distance < RENDER_DISTANCE {
                let test_x = self.player_x + ray_cos * distance;
                let test_y = self.player_y + ray_sin * distance;
                
                if self.is_wall(test_x, test_y) {
                    break;
                }
                distance += 1.0; // Higher precision
            }
            
            // Fish-eye correction
            distance *= (ray_angle - self.player_angle).cos();
            
            // Calculate wall height with perspective
            let wall_height = (screen_height * 0.6) / (distance / CELL_SIZE + 0.1);
            let wall_top = (screen_height / 2.0) - wall_height / 2.0;
            let wall_bottom = (screen_height / 2.0) + wall_height / 2.0;
            
            // Themed wall rendering with distance-based shading
            let brightness_factor = 1.0 - (distance / 500.0).min(1.0);
            let wall_color = self.current_theme.get_wall_color(&theme, brightness_factor, i);
            
            // Draw wall with thickness for better appearance
            let x = (i as f32 / num_rays as f32) * screen_width;
            let line_width = (screen_width / num_rays as f32).max(1.0);
            
            draw_rectangle(x, wall_top, line_width, wall_bottom - wall_top, wall_color);
            
            // Add wall edge highlighting for wireframe effect
            if i % 8 == 0 || distance < 100.0 {
                draw_line(x, wall_top, x, wall_bottom, 1.0, Color::from_rgba(255, 255, 255, 100));
            }
        }
        
        // Add screen flash effect for wall hits
        if self.wall_hit_flash > 0.0 {
            let flash_alpha = (self.wall_hit_flash * 100.0) as u8;
            draw_rectangle(0.0, 0.0, screen_width, screen_height, Color::from_rgba(255, 100, 100, flash_alpha));
        }
        
        // Draw enemies in 3D view
        self.draw_enemies_3d();
        
        // Draw professional crosshair
        self.draw_crosshair();
        
        // Enhanced HUD with professional styling
        self.draw_enhanced_hud();
    }
    
    fn draw_enhanced_hud(&self) {
        let screen_width = screen_width();
        let screen_height = screen_height();
        let theme = self.current_theme.get_config();
        
        // Themed HUD background
        let hud_height = 120.0;
        let hud_y = screen_height - hud_height;
        
        // Semi-transparent background with themed border
        draw_rectangle(0.0, hud_y, screen_width, hud_height, Color::from_rgba(0, 0, 0, 180));
        draw_rectangle_lines(0.0, hud_y, screen_width, hud_height, 2.0, theme.hud_primary);
        
        // FPS Counter with themed colors
        let fps_color = if self.fps_counter >= 60.0 {
            theme.hud_accent  // Good FPS
        } else if self.fps_counter >= 30.0 {
            theme.hud_secondary  // Okay FPS
        } else {
            Color::from_rgba(255, 0, 0, 255)  // Red for poor FPS
        };
        
        draw_text(&format!("FPS: {:.0}", self.fps_counter), 20.0, hud_y + 25.0, 20.0, fps_color);
        
        // Themed player info
        draw_text(&format!("PILOT: {}", self.username), 15.0, 55.0, 18.0, theme.text_primary);
        draw_text(&format!("LEVEL: {} | SCORE: {}", self.level, self.score), 15.0, 75.0, 18.0, theme.text_secondary);
        
        // Health and ammo bars
        let health_width = (self.health as f32 / 100.0) * 100.0;
        let ammo_width = (self.ammo as f32 / 30.0) * 100.0;
        
        // Themed health bar
        draw_text("HEALTH:", 15.0, 100.0, 16.0, theme.text_primary);
        draw_rectangle(80.0, 88.0, 100.0, 12.0, Color::from_rgba(100, 0, 0, 200));
        draw_rectangle(80.0, 88.0, health_width, 12.0, if self.health > 50 { theme.hud_accent } else { Color::from_rgba(255, 0, 0, 255) });
        draw_rectangle_lines(80.0, 88.0, 100.0, 12.0, 1.0, theme.hud_primary);
        
        // Themed ammo bar
        draw_text("AMMO:", 200.0, 100.0, 16.0, theme.text_primary);
        draw_rectangle(250.0, 88.0, 100.0, 12.0, Color::from_rgba(100, 100, 0, 200));
        draw_rectangle(250.0, 88.0, ammo_width, 12.0, if self.ammo > 10 { theme.hud_secondary } else { Color::from_rgba(255, 0, 0, 255) });
        draw_rectangle_lines(250.0, 88.0, 100.0, 12.0, 1.0, theme.hud_primary);
        
        // Themed mission status
        if self.game_won {
            let text = "🎉 MISSION COMPLETE! ALL LEVELS CLEARED! 🎉";
            let text_width = measure_text(text, None, 28, 1.0).width;
            draw_rectangle(screen_width/2.0 - text_width/2.0 - 10.0, screen_height/2.0 - 20.0, text_width + 20.0, 40.0, Color::from_rgba(0, 100, 0, 200));
            draw_text(text, screen_width/2.0 - text_width/2.0, screen_height/2.0, 28.0, theme.hud_accent);
        } else {
            let objective_text = self.current_theme.get_objective_text();
            draw_text(objective_text, 15.0, screen_height - 80.0, 18.0, theme.text_secondary);
        }
        
        // Themed controls help
        draw_rectangle(5.0, screen_height - 60.0, 450.0, 55.0, Color::from_rgba(0, 0, 0, 150));
        draw_rectangle_lines(5.0, screen_height - 60.0, 450.0, 55.0, 1.0, theme.hud_primary);
        draw_text("CONTROLS: WASD/Mouse=Move | SPACE=Shoot | T=Theme", 15.0, screen_height - 40.0, 16.0, theme.text_primary);
        draw_text("STATUS: Connected to Combat Network", 15.0, screen_height - 20.0, 16.0, theme.hud_accent);
    }
    
    fn draw_crosshair(&self) {
        let center_x = screen_width() / 2.0;
        let center_y = screen_height() / 2.0;
        let size = 15.0 + (self.crosshair_pulse.sin() * 3.0);
        let thickness = 2.0;
        let theme = self.current_theme.get_config();
        
        // Themed animated crosshair with pulse effect
        let alpha = if self.ammo > 0 { 200 } else { 100 };
        let color = if self.ammo > 0 { 
            Color::from_rgba((theme.hud_accent.r * 255.0) as u8, (theme.hud_accent.g * 255.0) as u8, (theme.hud_accent.b * 255.0) as u8, alpha) 
        } else { 
            Color::from_rgba(255, 0, 0, alpha) 
        };
        
        // Draw crosshair lines
        draw_line(center_x - size, center_y, center_x - 5.0, center_y, thickness, color);
        draw_line(center_x + 5.0, center_y, center_x + size, center_y, thickness, color);
        draw_line(center_x, center_y - size, center_x, center_y - 5.0, thickness, color);
        draw_line(center_x, center_y + 5.0, center_x, center_y + size, thickness, color);
        
        // Center dot
        draw_circle(center_x, center_y, 1.5, color);
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
        let map_size = 180.0;
        let map_x = screen_width() - map_size - 10.0;
        let map_y = 10.0;
        let cell_size = map_size / MAZE_WIDTH as f32;
        
        // Draw minimap background with enhanced styling
        draw_rectangle(map_x - 5.0, map_y - 25.0, map_size + 10.0, map_size + 30.0, Color::from_rgba(0, 0, 0, 180));
        draw_rectangle_lines(map_x - 5.0, map_y - 25.0, map_size + 10.0, map_size + 30.0, 2.0, Color::from_rgba(0, 255, 255, 150));
        draw_rectangle(map_x, map_y, map_size, map_size, Color::from_rgba(0, 0, 30, 200));
        draw_rectangle_lines(map_x, map_y, map_size, map_size, 2.0, BLUE);
        
        // Draw maze walls with better visibility
        for y in 0..MAZE_HEIGHT {
            for x in 0..MAZE_WIDTH {
                if self.maze[y][x] {
                    draw_rectangle(
                        map_x + x as f32 * cell_size,
                        map_y + y as f32 * cell_size,
                        cell_size,
                        cell_size,
                        Color::from_rgba(100, 150, 255, 255),
                    );
                }
            }
        }
        
        // Draw exit with pulsing effect
        let exit_map_x = map_x + (self.exit_x / CELL_SIZE) * cell_size;
        let exit_map_y = map_y + (self.exit_y / CELL_SIZE) * cell_size;
        let pulse = (self.crosshair_pulse * 2.0).sin() * 0.3 + 0.7;
        draw_rectangle(exit_map_x, exit_map_y, cell_size, cell_size, 
                      Color::from_rgba((255.0 * pulse) as u8, 0, 0, 255));
        
        // Draw player position and direction
        let player_map_x = map_x + (self.player_x / CELL_SIZE) * cell_size + cell_size / 2.0;
        let player_map_y = map_y + (self.player_y / CELL_SIZE) * cell_size + cell_size / 2.0;
        
        // Player dot with glow effect
        draw_circle(player_map_x, player_map_y, 6.0, Color::from_rgba(255, 255, 0, 100));
        draw_circle(player_map_x, player_map_y, 4.0, YELLOW);
        
        // Direction indicator
        let dir_length = 12.0;
        let end_x = player_map_x + self.player_angle.cos() * dir_length;
        let end_y = player_map_y + self.player_angle.sin() * dir_length;
        draw_line(player_map_x, player_map_y, end_x, end_y, 3.0, Color::from_rgba(255, 255, 0, 200));
        
        // Draw enemies on minimap
        for enemy in &self.enemies {
            if enemy.health > 0 {
                let enemy_map_x = map_x + (enemy.x / CELL_SIZE) * cell_size + cell_size / 2.0;
                let enemy_map_y = map_y + (enemy.y / CELL_SIZE) * cell_size + cell_size / 2.0;
                
                // Enemy dot (red for hostile)
                draw_circle(enemy_map_x, enemy_map_y, 3.0, RED);
                
                // Direction indicator for enemy
                let enemy_dir_length = 6.0;
                let enemy_end_x = enemy_map_x + enemy.angle.cos() * enemy_dir_length;
                let enemy_end_y = enemy_map_y + enemy.angle.sin() * enemy_dir_length;
                draw_line(enemy_map_x, enemy_map_y, enemy_end_x, enemy_end_y, 1.0, RED);
            }
        }
        
        // Enhanced minimap title
        draw_text("TACTICAL MAP", map_x, map_y - 8.0, 16.0, Color::from_rgba(0, 255, 255, 255));
    }
    
    fn draw_enemies_3d(&self) {
        let screen_width = screen_width();
        let screen_height = screen_height();
        
        for enemy in &self.enemies {
            if enemy.health <= 0 { continue; }
            
            // Calculate distance and angle to enemy
            let dx = enemy.x - self.player_x;
            let dy = enemy.y - self.player_y;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance > 500.0 { continue; } // Don't render distant enemies
            
            // Check if enemy is in field of view
            let angle_to_enemy = dy.atan2(dx);
            let angle_diff = angle_to_enemy - self.player_angle;
            let normalized_angle = ((angle_diff + PI) % (2.0 * PI)) - PI;
            
            if normalized_angle.abs() < FOV / 2.0 {
                // Check if enemy is visible (not behind walls)
                let steps = (distance / 5.0) as i32;
                let mut visible = true;
                
                for i in 1..steps {
                    let check_x = self.player_x + (dx / distance) * (i as f32 * 5.0);
                    let check_y = self.player_y + (dy / distance) * (i as f32 * 5.0);
                    if self.is_wall(check_x, check_y) {
                        visible = false;
                        break;
                    }
                }
                
                if visible {
                    // Calculate screen position
                    let screen_x = screen_width / 2.0 + (normalized_angle / FOV) * screen_width;
                    
                    // Enemy size based on distance (closer = bigger)
                    let enemy_size = (30.0 / (distance / 100.0)).min(50.0).max(5.0);
                    let enemy_y = screen_height / 2.0;
                    
                    // Draw enemy as an "eye" (classic Maze Wars style)
                    let eye_color = match enemy.state {
                        EnemyState::Patrolling => Color::from_rgba(255, 255, 0, 200), // Yellow
                        EnemyState::Chasing => Color::from_rgba(255, 100, 0, 255),    // Orange
                        EnemyState::Attacking => Color::from_rgba(255, 0, 0, 255),   // Red
                    };
                    
                    // Draw enemy eye
                    draw_circle(screen_x, enemy_y, enemy_size / 2.0, eye_color);
                    draw_circle(screen_x, enemy_y, enemy_size / 4.0, BLACK); // Pupil
                    
                    // Health bar above enemy
                    let health_ratio = enemy.health as f32 / 50.0;
                    let bar_width = enemy_size;
                    let bar_height = 4.0;
                    let bar_y = enemy_y - enemy_size / 2.0 - 10.0;
                    
                    draw_rectangle(screen_x - bar_width / 2.0, bar_y, bar_width, bar_height, Color::from_rgba(100, 0, 0, 200));
                    draw_rectangle(screen_x - bar_width / 2.0, bar_y, bar_width * health_ratio, bar_height, 
                                 if health_ratio > 0.5 { GREEN } else { RED });
                }
            }
        }
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