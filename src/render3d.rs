use crate::caster::cast_ray_topdown;
use crate::framebuffer::FrameBuffer;
use crate::maze::Maze;
use crate::player::Player;
use raylib::prelude::*;

fn wall_color(ch: char) -> Color {
    match ch {
        '#' => Color::GRAY,
        'A' => Color::BLUE,
        'B' => Color::MAROON,
        'C' => Color::ORANGE, // puerta CERRADA (bloquea)
        'E' => Color::LIME,   // salida normal (visible, no bloquea)
        'F' => Color::GOLD,   // final (visible, no bloquea)
        _ => Color::LIGHTGRAY,
    }
}

pub fn render3d(framebuffer: &mut FrameBuffer, maze: &Maze, player: &Player) -> Vec<f32> {
    let num_rays = framebuffer.width.max(1) as usize;
    let hw = framebuffer.width as f32 * 0.5;
    let hh = framebuffer.height as f32 * 0.5;
    let dist_to_proj = hw / (player.fov * 0.5).tan();

    // cielo
    framebuffer.set_color(Color::new(20, 24, 40, 255));
    for y in 0..(hh as i32) {
        for x in 0..framebuffer.width {
            framebuffer.set_pixel(x, y);
        }
    }
    // piso
    framebuffer.set_color(Color::new(30, 22, 18, 255));
    for y in (hh as i32)..framebuffer.height {
        for x in 0..framebuffer.width {
            framebuffer.set_pixel(x, y);
        }
    }

    let mut zbuffer = vec![f32::INFINITY; num_rays];

    for sx in 0..num_rays {
        let t = sx as f32 / num_rays as f32;
        let ray_angle = player.a - (player.fov * 0.5) + (player.fov * t);

        let hit = cast_ray_topdown(framebuffer, maze, player, ray_angle, false);

        // corrección ojo de pez
        let delta = (ray_angle - player.a).cos().abs().max(1e-6);
        let dist = hit.distance * delta;
        zbuffer[sx] = dist;

        let stake_h = (maze.block_size as f32 * dist_to_proj) / dist;
        let top = ((hh - stake_h * 0.5).max(0.0)) as i32;
        let bot = ((hh + stake_h * 0.5).min(framebuffer.height as f32 - 1.0)) as i32;

        let base = wall_color(hit.impact);
        let fade = (1.0 / (1.0 + dist * 0.002)).clamp(0.2, 1.0);
        let col = Color::new(
            (base.r as f32 * fade) as u8,
            (base.g as f32 * fade) as u8,
            (base.b as f32 * fade) as u8,
            255,
        );

        framebuffer.set_color(col);
        let x = sx as i32;
        for y in top..=bot {
            framebuffer.set_pixel(x, y);
        }
    }

    zbuffer
}
