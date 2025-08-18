use crate::framebuffer::FrameBuffer;
use crate::maze::Maze;
use crate::player::Player;
use raylib::prelude::*;

pub struct Hit {
    pub distance: f32,
    pub impact: char,
    pub hit_x: f32,
    pub hit_y: f32,
}

// Avanza y se detiene sólo al tocar un bloqueante (#, A, B)
pub fn cast_ray_topdown(
    framebuffer: &mut FrameBuffer,
    maze: &Maze,
    player: &Player,
    angle: f32,
    draw_line: bool,
) -> Hit {
    let mut d = 0.0_f32;
    let step = 2.0_f32;

    loop {
        let x = player.pos.x + angle.cos() * d;
        let y = player.pos.y + angle.sin() * d;

        // fuera de límites -> pared
        if x < 0.0
            || y < 0.0
            || x >= (maze.width * maze.block_size as usize) as f32
            || y >= (maze.height * maze.block_size as usize) as f32
        {
            return Hit {
                distance: d,
                impact: '#',
                hit_x: x,
                hit_y: y,
            };
        }

        let ci = (x / maze.block_size as f32) as i32;
        let cj = (y / maze.block_size as f32) as i32;
        let cell = maze.tile_at(ci, cj);

        if Maze::is_blocking(cell) {
            return Hit {
                distance: d,
                impact: cell,
                hit_x: x,
                hit_y: y,
            };
        }

        if draw_line {
            framebuffer.set_color(Color::WHITE);
            framebuffer.set_pixel(x as i32, y as i32);
        }

        d += step;
    }
}
