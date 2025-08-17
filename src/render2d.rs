use crate::draw_utils::draw_rect;
use crate::framebuffer::FrameBuffer;
use crate::maze::Maze;
use raylib::prelude::*;

fn color_for_cell(c: char) -> Color {
    match c {
        '#' => Color::DARKGRAY,
        'A' => Color::RED,
        'B' => Color::GREEN,
        'C' => Color::BLUE,
        'P' => Color::GOLD,
        'E' => Color::LIME,
        '.' | ' ' => Color::new(30, 30, 35, 255),
        _ => Color::LIGHTGRAY,
    }
}

pub fn render_maze_2d(framebuffer: &mut FrameBuffer, maze: &Maze) {
    // fondo
    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            framebuffer.set_color(Color::BLACK);
            framebuffer.set_pixel(x, y);
        }
    }

    let bs = maze.block_size;
    for (j, row) in maze.grid.iter().enumerate() {
        for (i, &cell) in row.iter().enumerate() {
            let xo = (i as i32) * bs;
            let yo = (j as i32) * bs;
            draw_rect(framebuffer, xo, yo, bs, bs, color_for_cell(cell));
        }
    }
}
