use raylib::prelude::*;
use std::collections::HashMap;

// Textura CPU para samplear por u,v
pub struct CpuTexture {
    pub w: i32,
    pub h: i32,
    pub pixels: ImageColors,
}
impl CpuTexture {
    pub fn from_path(path: &str) -> Self {
        let img = Image::load_image(path).expect(path);
        let (w, h) = (img.width, img.height);
        let pixels = img.get_image_data();
        Self { w, h, pixels }
    }
    #[inline]
    pub fn sample(&self, u: f32, v: f32) -> Color {
        let x = ((u.clamp(0.0, 0.9999) * self.w as f32) as i32).clamp(0, self.w - 1);
        let y = ((v.clamp(0.0, 0.9999) * self.h as f32) as i32).clamp(0, self.h - 1);
        self.pixels[(y * self.w + x) as usize]
    }
}

pub struct Textures {
    map: HashMap<char, CpuTexture>,
    fallback: CpuTexture,
}

impl Textures {
    pub fn new() -> Self {
        // patrón rosa/negro para detectar faltantes
        let mut img = Image::gen_image_color(64, 64, Color::MAGENTA);
        for y in 0..64 {
            for x in 0..64 {
                if ((x / 8) + (y / 8)) % 2 == 0 {
                    img.draw_pixel(x, y, Color::BLACK)
                }
            }
        }
        let (w, h) = (img.width, img.height);
        let pixels = img.get_image_data();
        Self {
            map: HashMap::new(),
            fallback: CpuTexture { w, h, pixels },
        }
    }

    pub fn insert(&mut self, key: char, path: &str) {
        self.map.insert(key, CpuTexture::from_path(path));
    }
    pub fn get(&self, key: char) -> &CpuTexture {
        self.map.get(&key).unwrap_or(&self.fallback)
    }

    // centraliza TODA la configuración
    pub fn load_default() -> Self {
        let mut t = Self::new();
        t.insert('#', "assets/wall_normal.png"); // muros
        t.insert('.', "assets/piso.png"); // piso (floor casting)
        t.insert('C', "assets/door.png"); // puertas (como pared)
        // t.insert('T', "assets/taylor_cursed.jpg"); // si quieres ver 'T' como “poster” en pared
        t.insert('M', "assets/taylor_enemy.png"); // <-- sprite del ENEMIGO (PNG con alpha)
        // t.insert('A', "..."); t.insert('B', "...");
        t.insert('E', "assets/stairs.png");
        //t.insert('F', "assets/final.png");
        t
    }
}
