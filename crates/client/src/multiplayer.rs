use macroquad::prelude::*;
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::time::Instant;
use ::rand::{Rng, thread_rng};

use crate::{OtherPlayer, Bullet, Notification, CELL_SIZE};

pub struct MultiplayerManager {
    pub socket: Option<UdpSocket>,
    pub other_players: HashMap<String, OtherPlayer>,
    pub bullets: Vec<Bullet>,
    pub notifications: Vec<Notification>,
    pub last_state_send: Instant,
    pub kills: i32,
    pub deaths: i32,
}

impl MultiplayerManager {
    pub fn new() -> Self {
        Self {
            socket: None,
            other_players: HashMap::new(),
            bullets: Vec::new(),
            notifications: Vec::new(),
            last_state_send: Instant::now(),
            kills: 0,
            deaths: 0,
        }
    }

    pub fn setup_networking(&mut self, server_addr: &str, username: &str) {
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            socket.set_nonblocking(true).ok();
            if let Ok(server_addr) = server_addr.parse::<SocketAddr>() {
                if socket.connect(&server_addr).is_ok() {
                    let connect_msg = format!("CONNECT:{}", username);
                    let _ = socket.send(connect_msg.as_bytes());
                    self.socket = Some(socket);
                }
            }
        }
    }

    pub fn get_random_spawn_point(&self, maze: &[[bool; 40]; 40], maze_width: usize, maze_height: usize) -> (f32, f32) {
        let mut rng = thread_rng();
        let mut attempts = 0;
        
        loop {
            attempts += 1;
            
            // Generate random position avoiding borders
            let x = rng.gen_range(2..(maze_width - 2));
            let y = rng.gen_range(2..(maze_height - 2));
            
            // Check if position is clear (not a wall)
            if !maze[y][x] {
                // Check surrounding area is also clear
                let mut area_clear = true;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let check_x = (x as i32 + dx) as usize;
                        let check_y = (y as i32 + dy) as usize;
                        if check_x < maze_width && check_y < maze_height && maze[check_y][check_x] {
                            area_clear = false;
                            break;
                        }
                    }
                    if !area_clear { break; }
                }
                
                if area_clear {
                    return (
                        (x as f32 + 0.5) * CELL_SIZE,
                        (y as f32 + 0.5) * CELL_SIZE
                    );
                }
            }
            
            // Fallback to default spawn after too many attempts
            if attempts > 100 {
                return (3.5 * CELL_SIZE, 3.5 * CELL_SIZE);
            }
        }
    }

    pub fn handle_network_messages(&mut self) -> Option<(f32, f32)> {
        let mut spawn_position = None;
        
        if let Some(socket) = &self.socket {
            let mut buf = [0; 1024];
            let mut messages = Vec::new();
            
            while let Ok((n, _)) = socket.recv_from(&mut buf) {
                if n > 0 {
                    let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                    messages.push(msg);
                }
            }
            
            for msg in messages {
                if let Some(pos) = self.process_network_message(&msg) {
                    spawn_position = Some(pos);
                }
            }
        }
        
        spawn_position
    }

    fn process_network_message(&mut self, msg: &str) -> Option<(f32, f32)> {
        if msg.starts_with("SPAWN:") {
            let parts: Vec<&str> = msg.split(':').collect();
            if parts.len() >= 3 {
                if let (Ok(x), Ok(y)) = (parts[1].parse::<f32>(), parts[2].parse::<f32>()) {
                    return Some((x, y));
                }
            }
        } else if msg.starts_with("STATE:") {
            let parts: Vec<&str> = msg.split(':').collect();
            if parts.len() >= 3 {
                let player_name = parts[1].to_string();
                let state_parts: Vec<&str> = parts[2].split(',').collect();
                if state_parts.len() >= 7 {
                    if let (Ok(x), Ok(y), Ok(angle), Ok(_level), Ok(health), Ok(_ammo), Ok(_score)) = (
                        state_parts[0].parse::<f32>(),
                        state_parts[1].parse::<f32>(),
                        state_parts[2].parse::<f32>(),
                        state_parts[3].parse::<i32>(),
                        state_parts[4].parse::<i32>(),
                        state_parts[5].parse::<i32>(),
                        state_parts[6].parse::<i32>(),
                    ) {
                        let other_player = OtherPlayer {
                            name: player_name.clone(),
                            x,
                            y,
                            angle,
                            health,
                            last_seen: Instant::now(),
                        };
                        self.other_players.insert(player_name.clone(), other_player);
                        println!("📍 Updated player {}: ({:.1}, {:.1}) angle={:.2} health={}", 
                                player_name, x, y, angle, health);
                    }
                }
            }
        } else if msg.starts_with("SHOOT:") {
            let parts: Vec<&str> = msg.split(':').collect();
            if parts.len() >= 3 {
                let shooter = parts[1].to_string();
                let shoot_parts: Vec<&str> = parts[2].split(',').collect();
                if shoot_parts.len() >= 4 {
                    if let (Ok(x), Ok(y), Ok(dx), Ok(dy)) = (
                        shoot_parts[0].parse::<f32>(),
                        shoot_parts[1].parse::<f32>(),
                        shoot_parts[2].parse::<f32>(),
                        shoot_parts[3].parse::<f32>(),
                    ) {
                        self.bullets.push(Bullet {
                            x,
                            y,
                            dx,
                            dy,
                            owner: shooter,
                            created: Instant::now(),
                        });
                    }
                }
            }
        } else if msg.starts_with("HIT:") {
            // Parse: HIT:victim:shooter:damage
            let parts: Vec<&str> = msg.split(':').collect();
            if parts.len() >= 4 {
                let victim = &parts[1];
                let shooter = &parts[2];
                let damage: i32 = parts[3].parse().unwrap_or(20);
                
                self.show_notification(&format!("{} hit {} for {} damage!", shooter, victim, damage), YELLOW);
            }
        } else if msg.starts_with("KILL:") {
            // Parse: KILL:victim:killer
            let parts: Vec<&str> = msg.split(':').collect();
            if parts.len() >= 3 {
                let victim = &parts[1];
                let killer = &parts[2];
                
                self.show_notification(&format!("💀 {} eliminated {}!", killer, victim), RED);
                self.kills += 1; // Increment kills if we're the killer
            }
        } else if msg.starts_with("DEATH:") {
            // Parse: DEATH:player
            let player_name = msg[6..].to_string();
            self.show_notification(&format!("💀 {} was eliminated!", player_name), ORANGE);
            self.deaths += 1; // Increment deaths if it's us
        } else if msg.starts_with("JOIN:") {
            let player_name = msg[5..].to_string();
            self.show_notification(&format!("🎮 {} joined the game!", player_name), GREEN);
        } else if msg.starts_with("LEAVE:") {
            let player_name = msg[6..].to_string();
            self.other_players.remove(&player_name);
            self.show_notification(&format!("👋 {} left the game!", player_name), GRAY);
        }
        
        None
    }

    pub fn send_player_state(&mut self, x: f32, y: f32, angle: f32, health: i32) {
        if let Some(ref socket) = self.socket {
            let now = Instant::now();
            if now.duration_since(self.last_state_send) > std::time::Duration::from_millis(50) {
                // Send state in format expected by server: STATE:x,y,angle,level,health,ammo,score
                let state_msg = format!("STATE:{:.1},{:.1},{:.2},{},{},{},{}", x, y, angle, 1, health, 30, 0);
                let _ = socket.send(state_msg.as_bytes());
                self.last_state_send = now;
            }
        }
    }

    pub fn send_shoot(&mut self, x: f32, y: f32, dx: f32, dy: f32) {
        if let Some(ref socket) = self.socket {
            let shoot_msg = format!("SHOOT:{},{},{},{}", x, y, dx, dy);
            let _ = socket.send(shoot_msg.as_bytes());
        }
    }

    pub fn send_hit(&mut self, victim: &str, damage: i32) {
        if let Some(ref socket) = self.socket {
            let hit_msg = format!("HIT:{}:{}", victim, damage);
            let _ = socket.send(hit_msg.as_bytes());
        }
    }

    pub fn show_notification(&mut self, message: &str, color: Color) {
        self.notifications.push(Notification {
            message: message.to_string(),
            color,
            created: Instant::now(),
        });
        
        if self.notifications.len() > 5 {
            self.notifications.remove(0);
        }
    }

    pub fn update_bullets(&mut self, dt: f32, player_x: f32, player_y: f32, username: &str, 
                         maze: &[[bool; 40]; 40], maze_width: usize, maze_height: usize) -> (bool, Option<String>) {
        let now = Instant::now();
        let mut hit_player = false;
        let mut hit_by_player = None;
        let mut hit_targets = Vec::new();
        
        self.bullets.retain_mut(|bullet| {
            if now.duration_since(bullet.created) > std::time::Duration::from_secs(3) {
                return false;
            }
            
            bullet.x += bullet.dx * dt;
            bullet.y += bullet.dy * dt;
            
            let gx = (bullet.x / CELL_SIZE) as usize;
            let gy = (bullet.y / CELL_SIZE) as usize;
            if gx >= maze_width || gy >= maze_height || maze[gy][gx] {
                return false;
            }
            
            // Check player collision
            if bullet.owner != username {
                let dist = ((bullet.x - player_x).powi(2) + (bullet.y - player_y).powi(2)).sqrt();
                if dist < 16.0 {
                    hit_player = true;
                    hit_by_player = Some(bullet.owner.clone());
                    return false;
                }
            }
            
            // Check other player collisions
            for (other_name, other_player) in &self.other_players {
                if bullet.owner != *other_name {
                    let dist = ((bullet.x - other_player.x).powi(2) + (bullet.y - other_player.y).powi(2)).sqrt();
                    if dist < 16.0 {
                        hit_targets.push(other_name.clone());
                        return false;
                    }
                }
            }
            
            true
        });
        
        // Send hit notifications after bullet processing
        for target in hit_targets {
            self.send_hit(&target, 20);
        }
        
        // Clean up old players
        let mut removed_players = Vec::new();
        self.other_players.retain(|name, player| {
            let should_keep = now.duration_since(player.last_seen) < std::time::Duration::from_secs(5);
            if !should_keep {
                removed_players.push(name.clone());
            }
            should_keep
        });
        
        for player_name in removed_players {
            self.show_notification(&format!("🔌 {} disconnected", player_name), ORANGE);
        }
        
        (hit_player, hit_by_player)
    }

    pub fn draw_other_players_on_minimap(&self, mx: f32, my: f32, cell: f32) {
        for player in self.other_players.values() {
            let player_color = if player.health > 75 {
                LIME
            } else if player.health > 50 {
                GREEN
            } else if player.health > 25 {
                YELLOW
            } else {
                RED
            };
            
            let px = mx + (player.x / CELL_SIZE) * cell;
            let py = my + (player.y / CELL_SIZE) * cell;
            
            // Pulsing effect for other players
            let pulse = (get_time() * 3.0).sin() * 0.3 + 0.7;
            let size = 3.5 * pulse as f32;
            
            draw_circle(px, py, size, player_color);
            draw_circle_lines(px, py, size, 1.0, WHITE);
            
            // Draw player name with background
            let name_width = measure_text(&player.name, None, 10, 1.0).width;
            draw_rectangle(px - name_width * 0.5 - 2.0, py - 15.0, name_width + 4.0, 12.0, 
                         Color::from_rgba(0, 0, 0, 150));
            draw_text(&player.name, px - name_width * 0.5, py - 6.0, 10.0, WHITE);
        }
    }

    pub fn draw_notifications(&mut self) {
        let now = Instant::now();
        
        self.notifications.retain(|notif| {
            now.duration_since(notif.created) < std::time::Duration::from_secs(5)
        });
        
        for (i, notif) in self.notifications.iter().enumerate() {
            let age = now.duration_since(notif.created).as_secs_f32();
            let alpha = (1.0 - age / 5.0).max(0.0);
            
            let y_pos = 100.0 + i as f32 * 25.0;
            let bg_color = Color::new(0.0, 0.0, 0.0, alpha * 0.8);
            let text_color = Color::new(notif.color.r, notif.color.g, notif.color.b, alpha);
            
            let text_width = measure_text(&notif.message, None, 16, 1.0).width;
            draw_rectangle(10.0, y_pos - 12.0, text_width + 20.0, 20.0, bg_color);
            draw_text(&notif.message, 20.0, y_pos, 16.0, text_color);
        }
    }

    pub fn get_player_count(&self) -> usize {
        self.other_players.len() + 1 // +1 for local player
    }

    pub fn get_score(&self) -> (i32, i32) {
        (self.kills, self.deaths)
    }
}
