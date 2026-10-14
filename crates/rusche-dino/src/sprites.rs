//! Game sprites from Wikimedia Commons (see `assets/README.md`).
//! PNGs are white silhouettes with alpha; the host tints them per scene color.

use std::collections::HashMap;

use egui::{Color32, ColorImage, Context, TextureHandle, TextureOptions, Vec2};

/// Virtual scene size (same numbers as `state.rsc`).
pub const SCENE_W: f32 = 600.0;
pub const SCENE_H: f32 = 150.0;
pub const SCALE: f32 = 2.0;

const SPRITE_FILES: &[(&str, &[u8])] = &[
    ("dino-run-0", include_bytes!("sprite_png/dino-run-0.png")),
    ("dino-run-1", include_bytes!("sprite_png/dino-run-1.png")),
    ("dino-jump", include_bytes!("sprite_png/dino-jump.png")),
    ("dino-duck-0", include_bytes!("sprite_png/dino-duck-0.png")),
    ("dino-duck-1", include_bytes!("sprite_png/dino-duck-1.png")),
    ("dino-dead", include_bytes!("sprite_png/dino-dead.png")),
    ("cactus-s", include_bytes!("sprite_png/cactus-s.png")),
    ("cactus-l", include_bytes!("sprite_png/cactus-l.png")),
    ("bird-0", include_bytes!("sprite_png/bird-0.png")),
    ("bird-1", include_bytes!("sprite_png/bird-1.png")),
    ("cloud", include_bytes!("sprite_png/cloud.png")),
    ("d0", include_bytes!("sprite_png/d0.png")),
    ("d1", include_bytes!("sprite_png/d1.png")),
    ("d2", include_bytes!("sprite_png/d2.png")),
    ("d3", include_bytes!("sprite_png/d3.png")),
    ("d4", include_bytes!("sprite_png/d4.png")),
    ("d5", include_bytes!("sprite_png/d5.png")),
    ("d6", include_bytes!("sprite_png/d6.png")),
    ("d7", include_bytes!("sprite_png/d7.png")),
    ("d8", include_bytes!("sprite_png/d8.png")),
    ("d9", include_bytes!("sprite_png/d9.png")),
    ("hi", include_bytes!("sprite_png/hi.png")),
    ("game-over", include_bytes!("sprite_png/game-over.png")),
];

pub struct SpriteAtlas {
    textures: HashMap<&'static str, TextureHandle>,
    sizes: HashMap<&'static str, Vec2>,
}

impl SpriteAtlas {
    pub fn load(ctx: &Context) -> Self {
        let mut textures = HashMap::new();
        let mut sizes = HashMap::new();
        for &(name, bytes) in SPRITE_FILES {
            let img = image::load_from_memory(bytes)
                .unwrap_or_else(|e| panic!("sprite `{name}`: {e}"))
                .to_rgba8();
            let size = [img.width() as usize, img.height() as usize];
            sizes.insert(name, Vec2::new(size[0] as f32, size[1] as f32));
            let color = ColorImage::from_rgba_unmultiplied(size, img.as_raw());
            let tex = ctx.load_texture(name, color, TextureOptions::NEAREST);
            textures.insert(name, tex);
        }
        Self { textures, sizes }
    }

    pub fn get(&self, name: &str) -> Option<(&TextureHandle, Vec2)> {
        let tex = self.textures.get(name)?;
        let size = *self.sizes.get(name)?;
        Some((tex, size))
    }
}

/// Tint for a white-alpha sprite (RGB from the scene command).
pub fn tint(r: f32, g: f32, b: f32) -> Color32 {
    Color32::from_rgb(
        r.clamp(0.0, 255.0) as u8,
        g.clamp(0.0, 255.0) as u8,
        b.clamp(0.0, 255.0) as u8,
    )
}
