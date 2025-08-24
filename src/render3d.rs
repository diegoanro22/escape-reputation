use crate::caster::cast_ray_topdown;
use crate::framebuffer::FrameBuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::textures::Textures;
use raylib::prelude::*;

#[inline]
fn fog_mix(mut c: Color, fog: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    c.r = (c.r as f32 * (1.0 - t) + fog.r as f32 * t) as u8;
    c.g = (c.g as f32 * (1.0 - t) + fog.g as f32 * t) as u8;
    c.b = (c.b as f32 * (1.0 - t) + fog.b as f32 * t) as u8;
    c
}

pub fn render3d(
    framebuffer: &mut FrameBuffer,
    maze: &Maze,
    player: &Player,
    textures: &Textures,
) -> Vec<f32> {
    let w = framebuffer.width as i32;
    let h = framebuffer.height as i32;
    let hw = w as f32 * 0.5;
    let hh = h as f32 * 0.5;
    let dist_to_proj = hw / (player.fov * 0.5).tan();
    let bs = maze.block_size as f32;

    // Colores base
    let sky = Color {
        r: 20,
        g: 24,
        b: 40,
        a: 255,
    };
    let floor = Color {
        r: 30,
        g: 22,
        b: 18,
        a: 255,
    };

    // Cielo “sólido” (lo podemos texturizar después si quieres)
    framebuffer.set_color(sky);
    for y in 0..(hh as i32) {
        for x in 0..w {
            framebuffer.set_pixel(x, y);
        }
    }

    // ====== FLOOR CASTING (piso texturizado para todas las celdas) ======
    // Sistema en "unidades de celdas" (no píxeles):
    let px = player.pos.x / bs;
    let py = player.pos.y / bs;

    // Dirección de la cámara y plano (perpendicular) de longitud tan(fov/2)
    let dirx = player.a.cos();
    let diry = player.a.sin();
    let plane_len = (player.fov * 0.5).tan();
    let planex = -diry * plane_len;
    let planey = dirx * plane_len;

    // Rayos del borde izquierdo y derecho de la pantalla
    let r0x = dirx - planex;
    let r0y = diry - planey;
    let r1x = dirx + planex;
    let r1y = diry + planey;

    // Textura del piso (usamos '.' como “suelo” global)
    let floor_tex = textures.get('.');

    // Recorremos filas desde el horizonte hacia abajo
    for sy in (hh as i32)..h {
        let p = sy as f32 - hh + 0.5; // distancia vertical en pantalla (en px)
        if p <= 0.0 {
            continue;
        }
        let pos_z = hh; // cámara a media altura de pantalla
        let row_dist = pos_z / p; // distancia en "celdas" a esa fila

        // Paso en X/Y al avanzar una columna en pantalla
        let step_x = row_dist * (r1x - r0x) / w as f32;
        let step_y = row_dist * (r1y - r0y) / w as f32;

        // Punto del mundo (en celdas) bajo la columna izquierda
        let mut world_x = px + row_dist * r0x;
        let mut world_y = py + row_dist * r0y;

        // Fog por distancia (aprox) – pasa a px multiplicando por tamaño de celda
        let dist_px = row_dist * bs;
        let fog_t = 1.0 - (-dist_px * 0.010).exp();

        for sx in 0..w {
            // u,v = fracción dentro de la celda
            let u = world_x.fract();
            let v = world_y.fract();

            let mut c = floor_tex.sample(u, v);
            c = fog_mix(c, sky, fog_t); // mezcla con cielo para profundidad

            framebuffer.set_color(c);
            framebuffer.set_pixel(sx, sy);

            world_x += step_x;
            world_y += step_y;
        }
    }
    // ====================================================================

    // (Opcional) si quieres un “fallback” de color bajo el piso texturizado:
    // framebuffer.set_color(floor);
    // for y in (hh as i32)..h { for x in 0..w { framebuffer.set_pixel(x, y); } }

    // ====== MUROS (igual que tenías, con texturas) ======
    let mut zbuffer = vec![f32::INFINITY; w as usize];

    for sx in 0..w as usize {
        let lerp = sx as f32 / (w as f32 - 1.0).max(1.0);
        let ray_angle = player.a - player.fov * 0.5 + player.fov * lerp;
        let ray_dx = ray_angle.cos();
        let ray_dy = ray_angle.sin();

        let hit = cast_ray_topdown(framebuffer, maze, player, ray_angle, false);

        // corrección ojo de pez
        let delta = (ray_angle - player.a).cos().abs().max(1e-6);
        let dist = hit.distance * delta;
        zbuffer[sx] = dist;

        let stake_h = (bs * dist_to_proj) / dist;
        let top = ((hh - stake_h * 0.5).max(0.0)) as i32;
        let bot = ((hh + stake_h * 0.5).min(h as f32 - 1.0)) as i32;

        let tex = textures.get(hit.impact);

        // decide cara vertical/horizontal según offset dentro de bloque
        let fx = hit.hit_x - ((hit.hit_x / bs).floor() * bs);
        let fy = hit.hit_y - ((hit.hit_y / bs).floor() * bs);
        let vertical = fx.min(bs - fx) < fy.min(bs - fy);

        // u en [0,1) estable
        let mut u = if vertical {
            (hit.hit_y / bs).fract()
        } else {
            (hit.hit_x / bs).fract()
        };
        if vertical && ray_dx > 0.0 {
            u = 1.0 - u;
        }
        if !vertical && ray_dy < 0.0 {
            u = 1.0 - u;
        }
        let u_eps = 0.5 / tex.w as f32;
        u = u.clamp(u_eps, 1.0 - u_eps);

        // sombreado + niebla suave
        let side_shade = if vertical { 0.82 } else { 1.0 };
        let fog_t_wall = 1.0 - (-dist * 0.010).exp();
        let fade = (1.0 / (1.0 + dist * 0.002)).clamp(0.3, 1.0);

        let denom = (bot - top).max(1) as f32;
        let v_eps = 0.5 / tex.h as f32;
        let x = sx as i32;

        for y in top..=bot {
            let mut v = (y - top) as f32 / denom;
            v = v.clamp(v_eps, 1.0 - v_eps);

            let mut c = tex.sample(u, v);
            c.r = (c.r as f32 * side_shade * fade) as u8;
            c.g = (c.g as f32 * side_shade * fade) as u8;
            c.b = (c.b as f32 * side_shade * fade) as u8;
            c = fog_mix(c, sky, fog_t_wall);

            framebuffer.set_color(c);
            framebuffer.set_pixel(x, y);
        }
    }

    zbuffer
}
