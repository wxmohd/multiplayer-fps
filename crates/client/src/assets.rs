use macroquad::prelude::*;
use std::collections::HashMap;
use crate::sprite_gen;

pub struct AssetManager {
    textures: HashMap<String, Texture2D>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    pub async fn load_texture(&mut self, name: &str, path: &str) -> Result<(), String> {
        match load_texture(path).await {
            Ok(texture) => {
                texture.set_filter(FilterMode::Nearest);
                self.textures.insert(name.to_string(), texture);
                Ok(())
            }
            Err(e) => Err(format!("Failed to load texture {}: {}", path, e)),
        }
    }

    pub fn get_texture(&self, name: &str) -> Option<&Texture2D> {
        self.textures.get(name)
    }

    pub async fn load_weapon_sprites(&mut self) -> Result<(), String> {
        // Generate sprites programmatically
        let modern_pistol_img = sprite_gen::create_modern_pistol();
        let flash_img = sprite_gen::create_muzzle_flash();

        // Convert to textures
        let modern_pistol_tex = Texture2D::from_image(&modern_pistol_img);
        let flash_tex = Texture2D::from_image(&flash_img);

        // Set filter and store
        modern_pistol_tex.set_filter(FilterMode::Nearest);
        flash_tex.set_filter(FilterMode::Nearest);

        self.textures.insert("modern_pistol".to_string(), modern_pistol_tex);
        self.textures.insert("muzzle_flash".to_string(), flash_tex);

        Ok(())
    }

    pub async fn load_ui_sprites(&mut self) -> Result<(), String> {
        // For now, skip UI sprites - we'll focus on weapons first
        Ok(())
    }
}

pub fn draw_sprite_centered(texture: &Texture2D, x: f32, y: f32, scale: f32, tint: Color) {
    let w = texture.width() * scale;
    let h = texture.height() * scale;
    draw_texture_ex(
        texture,
        x - w * 0.5,
        y - h * 0.5,
        tint,
        DrawTextureParams {
            dest_size: Some(Vec2::new(w, h)),
            ..Default::default()
        },
    );
}

pub fn draw_sprite_at(texture: &Texture2D, x: f32, y: f32, scale: f32, tint: Color) {
    let w = texture.width() * scale;
    let h = texture.height() * scale;
    draw_texture_ex(
        texture,
        x,
        y,
        tint,
        DrawTextureParams {
            dest_size: Some(Vec2::new(w, h)),
            ..Default::default()
        },
    );
}
