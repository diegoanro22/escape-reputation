use crate::framebuffer::FrameBuffer;
use raylib::prelude::*;

pub fn clear(framebuffer: &mut FrameBuffer, color: Color) {
    framebuffer.set_color(color);
    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            framebuffer.set_pixel(x, y);
        }
    }
}

pub fn draw_rect(framebuffer: &mut FrameBuffer, x0: i32, y0: i32, w: i32, h: i32, color: Color) {
    framebuffer.set_color(color);
    for y in y0..(y0 + h) {
        for x in x0..(x0 + w) {
            framebuffer.set_pixel(x, y);
        }
    }
}

pub fn draw_disc(framebuffer: &mut FrameBuffer, cx: i32, cy: i32, r: i32, color: Color) {
    framebuffer.set_color(color);
    for y in (cy - r)..=(cy + r) {
        for x in (cx - r)..=(cx + r) {
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= r * r {
                framebuffer.set_pixel(x, y);
            }
        }
    }
}

pub fn draw_centered_text(d: &mut RaylibDrawHandle, text: &str, y: i32, size: i32, color: Color) {
    let w = d.get_screen_width();
    let tw = d.measure_text(text, size);
    d.draw_text(text, (w - tw) / 2, y, size, color);
}
