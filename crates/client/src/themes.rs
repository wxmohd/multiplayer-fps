use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub enum LevelTheme {
    CandyMaze,
    Cyberpunk,
    MoroccanBazaar,
}

pub struct ThemeConfig {
    // Wall colors
    pub wall_primary: Color,
    pub wall_secondary: Color,
    pub wall_accent: Color,
    
    // Environment colors
    pub floor_color: Color,
    pub ceiling_color: Color,
    
    // HUD colors
    pub hud_primary: Color,
    pub hud_secondary: Color,
    pub hud_accent: Color,
    
    // UI text
    pub text_primary: Color,
    pub text_secondary: Color,
    
    // Effects
    pub glow_color: Color,
    pub particle_color: Color,
}

impl LevelTheme {
    pub fn from_level(level: usize) -> Self {
        match level {
            1 => LevelTheme::CandyMaze,
            2 => LevelTheme::Cyberpunk,
            3 => LevelTheme::MoroccanBazaar,
            _ => LevelTheme::CandyMaze,
        }
    }

    pub fn get_config(&self) -> ThemeConfig {
        match self {
            LevelTheme::CandyMaze => ThemeConfig {
                // Candy theme - sweet pastels
                wall_primary: Color::from_rgba(255, 182, 193, 255),    // Light pink
                wall_secondary: Color::from_rgba(255, 255, 255, 255),  // White
                wall_accent: Color::from_rgba(255, 105, 180, 255),     // Hot pink
                
                floor_color: Color::from_rgba(255, 192, 203, 255),     // Pink floor
                ceiling_color: Color::from_rgba(173, 216, 230, 255),   // Light blue sky
                
                hud_primary: Color::from_rgba(255, 20, 147, 255),      // Deep pink
                hud_secondary: Color::from_rgba(255, 182, 193, 255),   // Light pink
                hud_accent: Color::from_rgba(255, 255, 255, 255),      // White
                
                text_primary: Color::from_rgba(139, 69, 19, 255),      // Chocolate brown
                text_secondary: Color::from_rgba(255, 20, 147, 255),   // Deep pink
                
                glow_color: Color::from_rgba(255, 182, 193, 150),      // Pink glow
                particle_color: Color::from_rgba(255, 255, 255, 200),  // White sparkles
            },
            
            LevelTheme::Cyberpunk => ThemeConfig {
                // Cyberpunk theme - neon and dark
                wall_primary: Color::from_rgba(25, 25, 50, 255),       // Dark blue
                wall_secondary: Color::from_rgba(50, 25, 75, 255),     // Dark purple
                wall_accent: Color::from_rgba(255, 0, 255, 255),       // Magenta neon
                
                floor_color: Color::from_rgba(20, 20, 40, 255),        // Dark floor
                ceiling_color: Color::from_rgba(10, 10, 30, 255),      // Very dark ceiling
                
                hud_primary: Color::from_rgba(255, 0, 255, 255),       // Magenta
                hud_secondary: Color::from_rgba(0, 255, 255, 255),     // Cyan
                hud_accent: Color::from_rgba(255, 255, 255, 255),      // White
                
                text_primary: Color::from_rgba(0, 255, 255, 255),      // Cyan
                text_secondary: Color::from_rgba(255, 0, 255, 255),    // Magenta
                
                glow_color: Color::from_rgba(255, 0, 255, 100),        // Magenta glow
                particle_color: Color::from_rgba(0, 255, 255, 150),    // Cyan particles
            },
            
            LevelTheme::MoroccanBazaar => ThemeConfig {
                // Moroccan theme - warm earth tones
                wall_primary: Color::from_rgba(205, 133, 63, 255),     // Peru/tan
                wall_secondary: Color::from_rgba(160, 82, 45, 255),    // Saddle brown
                wall_accent: Color::from_rgba(0, 128, 128, 255),       // Teal accent
                
                floor_color: Color::from_rgba(139, 69, 19, 255),       // Saddle brown
                ceiling_color: Color::from_rgba(222, 184, 135, 255),   // Burlywood
                
                hud_primary: Color::from_rgba(218, 165, 32, 255),      // Goldenrod
                hud_secondary: Color::from_rgba(160, 82, 45, 255),     // Saddle brown
                hud_accent: Color::from_rgba(0, 128, 128, 255),        // Teal
                
                text_primary: Color::from_rgba(139, 69, 19, 255),      // Saddle brown
                text_secondary: Color::from_rgba(218, 165, 32, 255),   // Goldenrod
                
                glow_color: Color::from_rgba(255, 215, 0, 100),        // Gold glow
                particle_color: Color::from_rgba(218, 165, 32, 150),   // Golden particles
            },
        }
    }

    pub fn get_minimap_title(&self) -> &'static str {
        match self {
            LevelTheme::CandyMaze => "CANDY MAP",
            LevelTheme::Cyberpunk => "TACTICAL SCAN",
            LevelTheme::MoroccanBazaar => "BAZAAR MAP",
        }
    }

    pub fn get_objective_text(&self) -> &'static str {
        match self {
            LevelTheme::CandyMaze => "🍭 OBJECTIVE: Find the candy exit!",
            LevelTheme::Cyberpunk => "🎯 OBJECTIVE: Hack the exit portal",
            LevelTheme::MoroccanBazaar => "🏺 OBJECTIVE: Escape through the archway",
        }
    }

    pub fn get_wall_color(&self, config: &ThemeConfig, brightness: f32, ray_index: usize) -> Color {
        match self {
            LevelTheme::CandyMaze => {
                // Candy cane stripes
                let base_color = if ray_index % 16 < 8 {
                    config.wall_primary
                } else {
                    config.wall_secondary
                };
                Color::from_rgba(
                    (base_color.r * brightness) as u8,
                    (base_color.g * brightness) as u8,
                    (base_color.b * brightness) as u8,
                    255
                )
            },
            LevelTheme::Cyberpunk => {
                // Neon-lit walls with occasional accent lines
                let base_color = if ray_index % 32 == 0 {
                    config.wall_accent // Neon accent lines
                } else {
                    config.wall_primary
                };
                Color::from_rgba(
                    (base_color.r * brightness) as u8,
                    (base_color.g * brightness) as u8,
                    (base_color.b * brightness) as u8,
                    255
                )
            },
            LevelTheme::MoroccanBazaar => {
                // Geometric patterns
                let pattern = (ray_index / 4) % 3;
                let base_color = match pattern {
                    0 => config.wall_primary,
                    1 => config.wall_secondary,
                    _ => config.wall_accent,
                };
                Color::from_rgba(
                    (base_color.r * brightness) as u8,
                    (base_color.g * brightness) as u8,
                    (base_color.b * brightness) as u8,
                    255
                )
            },
        }
    }

    pub fn get_enemy_color(&self, enemy_state: &crate::EnemyState) -> Color {
        match self {
            LevelTheme::CandyMaze => {
                // Candy-themed enemies
                match enemy_state {
                    crate::EnemyState::Patrolling => Color::from_rgba(255, 182, 193, 255), // Pink gummy
                    crate::EnemyState::Chasing => Color::from_rgba(255, 105, 180, 255),    // Hot pink
                    crate::EnemyState::Attacking => Color::from_rgba(255, 20, 147, 255),   // Deep pink
                }
            },
            LevelTheme::Cyberpunk => {
                // Cyber-themed enemies with neon colors
                match enemy_state {
                    crate::EnemyState::Patrolling => Color::from_rgba(0, 255, 255, 255),   // Cyan
                    crate::EnemyState::Chasing => Color::from_rgba(255, 255, 0, 255),      // Yellow
                    crate::EnemyState::Attacking => Color::from_rgba(255, 0, 255, 255),    // Magenta
                }
            },
            LevelTheme::MoroccanBazaar => {
                // Moroccan-themed enemies with warm colors
                match enemy_state {
                    crate::EnemyState::Patrolling => Color::from_rgba(218, 165, 32, 255),  // Goldenrod
                    crate::EnemyState::Chasing => Color::from_rgba(205, 133, 63, 255),     // Peru
                    crate::EnemyState::Attacking => Color::from_rgba(160, 82, 45, 255),    // Saddle brown
                }
            },
        }
    }

    pub fn draw_atmospheric_effects(&self, config: &ThemeConfig, crosshair_pulse: f32) {
        let screen_width = screen_width();
        let screen_height = screen_height();
        
        match self {
            LevelTheme::CandyMaze => {
                // Floating sparkles
                for i in 0..20 {
                    let x = (i as f32 * 47.3 + crosshair_pulse * 20.0) % screen_width;
                    let y = (i as f32 * 31.7 + crosshair_pulse * 15.0) % screen_height;
                    let size = 2.0 + (crosshair_pulse + i as f32).sin() * 1.0;
                    draw_circle(x, y, size, config.particle_color);
                }
            },
            LevelTheme::Cyberpunk => {
                // Scanlines and digital rain effect
                for i in 0..10 {
                    let y = (i as f32 * screen_height / 10.0) + (crosshair_pulse * 2.0) % 4.0;
                    draw_line(0.0, y, screen_width, y, 1.0, Color::from_rgba(0, 255, 255, 30));
                }
                
                // Digital rain
                for i in 0..15 {
                    let x = (i as f32 * screen_width / 15.0);
                    let y = (crosshair_pulse * 100.0 + i as f32 * 50.0) % (screen_height + 100.0);
                    draw_rectangle(x, y, 2.0, 20.0, Color::from_rgba(0, 255, 0, 100));
                }
            },
            LevelTheme::MoroccanBazaar => {
                // Warm ambient lighting and dust particles
                for i in 0..25 {
                    let x = (i as f32 * 23.1 + crosshair_pulse * 5.0) % screen_width;
                    let y = (i as f32 * 17.9 + crosshair_pulse * 3.0) % screen_height;
                    let size = 1.0 + (crosshair_pulse * 0.5 + i as f32).sin() * 0.5;
                    draw_circle(x, y, size, Color::from_rgba(218, 165, 32, 80));
                }
            },
        }
    }
}
