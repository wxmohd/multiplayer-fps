use macroquad::prelude::*;

// Simple 2D front-facing pistol view (what player sees looking down)
pub fn create_modern_pistol() -> Image {
    let w = 60u16;
    let h = 80u16;
    let mut img = Image::gen_image_color(w, h, Color::new(0.0, 0.0, 0.0, 0.0));
    
    let gun_metal = Color::from_rgba(80, 85, 95, 255);
    let dark_metal = Color::from_rgba(50, 55, 65, 255);
    let light_metal = Color::from_rgba(110, 115, 125, 255);
    let grip_texture = Color::from_rgba(35, 40, 45, 255);
    
    // Main body - simple rectangular shape (front view)
    for y in 15..45 {
        for x in 20..40 {
            img.set_pixel(x as u32, y as u32, gun_metal);
        }
    }
    
    // Barrel end (what you see looking down the barrel)
    for y in 10..15 {
        for x in 25..35 {
            img.set_pixel(x as u32, y as u32, dark_metal);
        }
    }
    
    // Barrel opening (small dark circle)
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx*dx + dy*dy <= 1 {
                img.set_pixel((30 + dx) as u32, (12 + dy) as u32, Color::from_rgba(20, 20, 25, 255));
            }
        }
    }
    
    // Front sight (small rectangle at front)
    for y in 8..12 {
        for x in 28..32 {
            img.set_pixel(x as u32, y as u32, light_metal);
        }
    }
    
    // Rear sight (wider rectangle at back)
    for y in 15..18 {
        for x in 22..38 {
            img.set_pixel(x as u32, y as u32, light_metal);
        }
    }
    // Rear sight notch (small gap in middle)
    for x in 28..32 {
        img.set_pixel(x as u32, 16, dark_metal);
    }
    
    // Grip (below main body)
    for y in 45..70 {
        for x in 22..38 {
            img.set_pixel(x as u32, y as u32, grip_texture);
        }
    }
    
    // Grip texture (simple crosshatch pattern)
    for y in 48..67 {
        for x in 24..36 {
            if (x + y) % 4 == 0 {
                img.set_pixel(x as u32, y as u32, dark_metal);
            }
        }
    }
    
    // Trigger guard (simple outline)
    for x in 18..22 {
        img.set_pixel(x as u32, 45, gun_metal);
        img.set_pixel(x as u32, 55, gun_metal);
    }
    for y in 45..55 {
        img.set_pixel(18, y as u32, gun_metal);
        img.set_pixel(21, y as u32, gun_metal);
    }
    
    // Trigger (small rectangle inside guard)
    for y in 48..52 {
        for x in 15..18 {
            img.set_pixel(x as u32, y as u32, light_metal);
        }
    }
    
    img
}


pub fn create_muzzle_flash() -> Image {
    let size = 16u16;
    let mut img = Image::gen_image_color(size, size, Color::new(0.0, 0.0, 0.0, 0.0));
    
    let flash_color = Color::from_rgba(255, 240, 200, 200);
    
    // Simple triangle
    for y in 0..size {
        let width = (size - y) / 4;
        let center = size / 2;
        for x in (center - width)..(center + width) {
            if x < size && y < size {
                img.set_pixel(x as u32, y as u32, flash_color);
            }
        }
    }
    
    img
}
