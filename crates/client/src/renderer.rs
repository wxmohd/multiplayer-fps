use macroquad::prelude::*;
use crate::assets::AssetManager;

pub struct Renderer {
    pub frame_count: u32,
    pub last_fps_time: f64,
    pub fps: f32,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            last_fps_time: get_time(),
            fps: 0.0,
        }
    }

    pub fn update_fps(&mut self) {
        self.frame_count += 1;
        let current_time = get_time();
        
        if current_time - self.last_fps_time >= 1.0 {
            self.fps = self.frame_count as f32 / (current_time - self.last_fps_time) as f32;
            self.frame_count = 0;
            self.last_fps_time = current_time;
        }
    }

    // Vibrant, distinct wall colors for each level
    pub fn get_wall_color(level: u32, brightness: f32) -> Color {
        let base_color = match level {
            1 => Color::from_rgba(139, 69, 19, 255),    // Saddle brown - earthy maze
            2 => Color::from_rgba(25, 25, 112, 255),    // Midnight blue - deep ocean
            3 => Color::from_rgba(128, 0, 128, 255),    // Purple - mystical cavern
            _ => Color::from_rgba(128, 128, 128, 255),  // Gray
        };
        
        Color::new(
            (base_color.r * brightness).min(1.0),
            (base_color.g * brightness).min(1.0),
            (base_color.b * brightness).min(1.0),
            base_color.a,
        )
    }

    // Themed floor colors matching each level
    pub fn get_floor_color(level: u32) -> Color {
        match level {
            1 => Color::from_rgba(210, 180, 140, 255), // Tan - sandy ground
            2 => Color::from_rgba(0, 100, 100, 255),   // Dark cyan - ocean floor
            3 => Color::from_rgba(75, 0, 130, 255),    // Indigo - mystical floor
            _ => Color::from_rgba(128, 128, 128, 255), // Gray
        }
    }

    // Atmospheric sky colors for each level theme
    pub fn get_sky_color(level: u32) -> Color {
        match level {
            1 => Color::from_rgba(255, 218, 185, 255), // Peach puff - desert sky
            2 => Color::from_rgba(0, 191, 255, 255),   // Deep sky blue - underwater
            3 => Color::from_rgba(72, 61, 139, 255),   // Dark slate blue - mystical
            _ => Color::from_rgba(100, 149, 237, 255), // Cornflower blue
        }
    }

    // Draw simple, clean weapon
    pub fn draw_weapon(
        level: u32,
        fire_t: f32,
        recoil: f32,
        bob_phase: f32,
        ammo: i32,
        assets: &AssetManager,
    ) {
        let sw = screen_width();
        let sh = screen_height();
        
        let weapon_x = sw * 0.5;
        let weapon_y = sh * 0.85;
        
        // Weapon sway and recoil
        let sway_x = bob_phase.sin() * 8.0;
        let sway_y = bob_phase.cos() * 4.0 + recoil * -25.0;
        
        // Fire animation
        let fire_offset = fire_t * 15.0;
        
        // Draw weapon sprite based on level
        let weapon_texture = match level {
            1 => assets.get_texture("modern_pistol"),
            2 => assets.get_texture("modern_pistol"),
            3 => assets.get_texture("modern_pistol"),
            _ => assets.get_texture("modern_pistol"),
        };
        
        if let Some(texture) = weapon_texture {
            draw_texture_ex(
                texture,
                weapon_x + sway_x - texture.width() * 3.0,
                weapon_y + sway_y + fire_offset - texture.height() * 3.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(texture.width() * 6.0, texture.height() * 6.0)),
                    ..Default::default()
                },
            );
        }

        // Muzzle flash when firing
        if fire_t > 0.01 {
            if let Some(flash_tex) = assets.get_texture("muzzle_flash") {
                let flash_x = weapon_x + sway_x + 80.0;
                let flash_y = weapon_y + sway_y + fire_offset - 60.0;
                let flash_scale = 3.0 + 4.0 * fire_t;
                let flash_alpha = (1.0 - fire_t * 0.7).max(0.3);
                
                draw_texture_ex(
                    flash_tex,
                    flash_x - flash_tex.width() * flash_scale * 0.5,
                    flash_y - flash_tex.height() * flash_scale * 0.5,
                    Color::new(1.0, 1.0, 1.0, flash_alpha),
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(
                            flash_tex.width() * flash_scale,
                            flash_tex.height() * flash_scale,
                        )),
                        ..Default::default()
                    },
                );
            }
        }

        // Ammo counter
        let ammo_text = format!("AMMO: {}", ammo);
        draw_text(&ammo_text, sw - 150.0, sh - 30.0, 24.0, WHITE);
    }

    // Draw HUD elements
    pub fn draw_hud(&self, level: u32, health: i32, score: i32) {
        let sw = screen_width();
        
        // Level indicator
        draw_text(&format!("LEVEL: {}", level), 20.0, 30.0, 24.0, WHITE);
        
        // Health
        draw_text(&format!("HEALTH: {}", health), 20.0, 60.0, 24.0, WHITE);
        
        // Score
        draw_text(&format!("SCORE: {}", score), 20.0, 90.0, 24.0, WHITE);
        
        // FPS counter
        draw_text(&format!("FPS: {:.1}", self.fps), sw - 100.0, 30.0, 20.0, WHITE);
        
        // Crosshair
        let center_x = sw * 0.5;
        let center_y = screen_height() * 0.5;
        let crosshair_size = 10.0;
        
        draw_line(
            center_x - crosshair_size, center_y,
            center_x + crosshair_size, center_y,
            2.0, WHITE
        );
        draw_line(
            center_x, center_y - crosshair_size,
            center_x, center_y + crosshair_size,
            2.0, WHITE
        );
    }

    // Draw minimap
    pub fn draw_minimap(&self, player_x: f32, player_y: f32, player_angle: f32, maze: &[[bool; 32]; 32]) {
        let sw = screen_width();
        let sh = screen_height();
        
        let minimap_size = 150.0;
        let minimap_x = sw - minimap_size - 20.0;
        let minimap_y = 20.0;
        
        // Minimap background
        draw_rectangle(minimap_x, minimap_y, minimap_size, minimap_size, Color::from_rgba(0, 0, 0, 180));
        draw_rectangle_lines(minimap_x, minimap_y, minimap_size, minimap_size, 2.0, WHITE);
        
        // Draw maze
        let cell_size = minimap_size / 32.0;
        for y in 0..32 {
            for x in 0..32 {
                if maze[y][x] {
                    draw_rectangle(
                        minimap_x + x as f32 * cell_size,
                        minimap_y + y as f32 * cell_size,
                        cell_size,
                        cell_size,
                        Color::from_rgba(100, 100, 100, 255)
                    );
                }
            }
        }
        
        // Draw player position
        let player_minimap_x = minimap_x + (player_x / 32.0) * minimap_size;
        let player_minimap_y = minimap_y + (player_y / 32.0) * minimap_size;
        
        draw_circle(player_minimap_x, player_minimap_y, 3.0, RED);
        
        // Draw player direction
        let dir_x = player_minimap_x + player_angle.cos() * 8.0;
        let dir_y = player_minimap_y + player_angle.sin() * 8.0;
        draw_line(player_minimap_x, player_minimap_y, dir_x, dir_y, 2.0, RED);
    }
}
