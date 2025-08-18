// src/render3d.rs
use crate::caster::cast_ray_topdown;
use crate::framebuffer::FrameBuffer;
use crate::maze::Maze;
use crate::player::Player;
use raylib::prelude::*;
use std::f32::consts::PI;

fn wall_color(ch: char) -> Color {
    match ch {
        'A' => Color::RED,
        'B' => Color::GREEN,
        'C' => Color::BLUE,
        'E' => Color::BLUE,
        '+' => Color::YELLOW,
        '-' => Color::ORANGE,
        '|' => Color::PURPLE,
        'g' => Color::SKYBLUE,
        '#' => Color::GRAY,
        _ => Color::LIGHTGRAY,
    }
}

/// Renderiza el mundo 3D en el framebuffer y devuelve un z-buffer (distancia por columna)
pub fn render3d(framebuffer: &mut FrameBuffer, maze: &Maze, player: &Player) -> Vec<f32> {
    let num_rays = framebuffer.width.max(1) as usize;
    let hw = framebuffer.width as f32 * 0.5;
    let hh = framebuffer.height as f32 * 0.5;

    // Distancia al plano de proyección (clásico)
    let dist_to_proj = hw / (player.fov * 0.5).tan();

    // Cielo
    framebuffer.set_color(Color::new(20, 24, 40, 255));
    for y in 0..(hh as i32) {
        for x in 0..framebuffer.width {
            framebuffer.set_pixel(x, y);
        }
    }
    // Piso
    framebuffer.set_color(Color::new(30, 22, 18, 255));
    for y in (hh as i32)..framebuffer.height {
        for x in 0..framebuffer.width {
            framebuffer.set_pixel(x, y);
        }
    }

    let mut zbuffer = vec![f32::INFINITY; num_rays];

    // Barrido de rayos a lo largo del FOV
    for sx in 0..num_rays {
        let t = sx as f32 / num_rays as f32;
        let ray_angle = player.a - (player.fov * 0.5) + (player.fov * t);

        // rayo en top-down (ya validado)
        let hit = cast_ray_topdown(framebuffer, maze, player, ray_angle, false);

        // corrección de ojo de pez
        let delta = (ray_angle - player.a).cos().abs().max(1e-6);
        let dist = hit.distance * delta;
        zbuffer[sx] = dist;

        // altura de la columna
        let stake_h = (maze.block_size as f32 * dist_to_proj) / dist;

        // top / bottom en pantalla
        let top = ((hh - stake_h * 0.5).max(0.0)) as i32;
        let bot = ((hh + stake_h * 0.5).min(framebuffer.height as f32 - 1.0)) as i32;

        // sombreado por distancia (barato)
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

pub fn draw_exit_markers(
    framebuffer: &mut FrameBuffer,
    maze: &Maze,
    player: &Player,
    zbuffer: &[f32],
) {
    let hw = framebuffer.width as f32 * 0.5;
    let hh = framebuffer.height as f32 * 0.5;
    let dist_to_proj = hw / (player.fov * 0.5).tan();

    for (j, row) in maze.grid.iter().enumerate() {
        for (i, &cell) in row.iter().enumerate() {
            if cell != 'E' {
                continue;
            }

            // centro mundial (pixeles) de la celda E
            let wx = (i as f32 + 0.5) * maze.block_size as f32;
            let wy = (j as f32 + 0.5) * maze.block_size as f32;

            let dx = wx - player.pos.x;
            let dy = wy - player.pos.y;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);

            // ángulo hacia la salida y relación con mirada
            let mut rel = dy.atan2(dx) - player.a;
            while rel > PI {
                rel -= 2.0 * PI;
            }
            while rel < -PI {
                rel += 2.0 * PI;
            }

            let half_fov = player.fov * 0.5;
            if rel.abs() > half_fov + 0.25 {
                continue;
            } // fuera de pantalla

            // proyección en pantalla
            let cx = hw + (rel / player.fov) * framebuffer.width as f32;
            let sprite_h = (maze.block_size as f32 * dist_to_proj) / dist * 1.2;
            let sprite_w = sprite_h * 0.5; // pilar delgado

            let x0 = (cx - sprite_w * 0.5).floor().max(0.0) as i32;
            let x1 = (cx + sprite_w * 0.5)
                .ceil()
                .min((framebuffer.width - 1) as f32) as i32;
            let top = (hh - sprite_h).max(0.0) as i32;
            let bot = (hh + sprite_h).min((framebuffer.height - 1) as f32) as i32;

            for x in x0..=x1 {
                let col = x as usize;
                if col >= zbuffer.len() {
                    break;
                }

                // respeta paredes: solo pinta si la salida está más cerca que la pared
                if dist < zbuffer[col] {
                    // brillo decente con atenuación
                    let fade = (1.0 / (1.0 + dist * 0.02)).clamp(0.25, 1.0);
                    let r = (80.0 * fade) as u8; // un verde con toques amarillos
                    let g = (255.0 * fade) as u8;
                    let b = (90.0 * fade) as u8;
                    framebuffer.set_color(Color::new(r, g, b, 255));

                    for y in top..=bot {
                        framebuffer.set_pixel(x, y);
                    }
                }
            }
        }
    }
}
