use raylib::prelude::*;

pub struct FrameBuffer {
    pub width: i32,
    pub height: i32,
    pub color_buffer: Image,
    background_color: Color,
    current_color: Color,
}

impl FrameBuffer {
    pub fn new(width: i32, height: i32, background_color: Color) -> Self {
        let color_buffer = Image::gen_image_color(width, height, background_color);
        FrameBuffer {
            width,
            height,
            color_buffer,
            background_color,
            current_color: Color::WHITE,
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.color_buffer = Image::gen_image_color(self.width, self.height, color);
    }

    pub fn clear(&mut self) {
        self.color_buffer = Image::gen_image_color(self.width, self.height, self.current_color);
    }

    pub fn set_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn set_pixel(&mut self, x: i32, y: i32) {
        if (0..self.width).contains(&x) && (0..self.height).contains(&y) {
            self.color_buffer.draw_pixel(x, y, self.current_color);
        }
    }

    pub fn render_to_file(&self, file_path: &str) -> Result<(), String> {
        self.color_buffer.export_image(file_path);
        Ok(())
    }
}
