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

        // corrección fish-eye
        let delta = (ray_angle - player.a).cos().abs().max(1e-6);
        let dist = hit.distance * delta;
        zbuffer[sx] = dist;

        if !Maze::is_blocking(hit.impact) {
            continue;
        }

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

// Bloques visibles para E, C y F (igual ancho/alto que una pared)
pub fn draw_markers_as_blocks(
    framebuffer: &mut FrameBuffer,
    maze: &Maze,
    player: &Player,
    zbuffer: &[f32],
) {
    use std::f32::consts::PI;

    let hw = framebuffer.width as f32 * 0.5;
    let hh = framebuffer.height as f32 * 0.5;
    let dist_to_proj = hw / (player.fov * 0.5).tan();

    for (j, row) in maze.grid.iter().enumerate() {
        for (i, &cell) in row.iter().enumerate() {
            if !(cell == 'E' || cell == 'C' || cell == 'F') {
                continue;
            }

            let wx = (i as f32 + 0.5) * maze.block_size as f32;
            let wy = (j as f32 + 0.5) * maze.block_size as f32;

            let dx = wx - player.pos.x;
            let dy = wy - player.pos.y;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);

            let mut rel = dy.atan2(dx) - player.a;
            while rel > PI {
                rel -= 2.0 * PI;
            }
            while rel < -PI {
                rel += 2.0 * PI;
            }

            let half_fov = player.fov * 0.5;
            if rel.abs() > half_fov + 0.35 {
                continue;
            }

            let block_h = (maze.block_size as f32 * dist_to_proj) / dist;
            let top = (hh - block_h * 0.5).max(0.0) as i32;
            let bot = (hh + block_h * 0.5).min(framebuffer.height as f32 - 1.0) as i32;

            let block_w = (maze.block_size as f32 * dist_to_proj) / dist;
            let cx = hw + (rel / player.fov) * framebuffer.width as f32;
            let thickness = if cell == 'C' { 0.9 } else { 1.0 };
            let half_w = (block_w * thickness * 0.5).max(1.0);

            let x0 = (cx - half_w).floor().max(0.0) as i32;
            let x1 = (cx + half_w).ceil().min((framebuffer.width - 1) as f32) as i32;

            let (r, g, b) = match cell {
                'E' => (80u8, 255u8, 90u8),  // escaleras: verde
                'C' => (255u8, 170u8, 40u8), // puerta: ámbar
                'F' => (255u8, 215u8, 0u8),  // final: dorado
                _ => (200u8, 200u8, 200u8),
            };

            for x in x0..=x1 {
                let col = x as usize;
                if col >= zbuffer.len() {
                    break;
                }
                if dist < zbuffer[col] {
                    let fade = (1.0 / (1.0 + dist * 0.002)).clamp(0.3, 1.0);
                    framebuffer.set_color(Color::new(
                        (r as f32 * fade) as u8,
                        (g as f32 * fade) as u8,
                        (b as f32 * fade) as u8,
                        255,
                    ));
                    for y in top..=bot {
                        framebuffer.set_pixel(x, y);
                    }
                }
            }
        }
    }
}
