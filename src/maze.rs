use crate::framebuffer::FrameBuffer;
use raylib::prelude::*;
use std::fs;

pub struct Maze {
    pub grid: Vec<Vec<char>>,
    pub width: usize,
    pub height: usize,
    pub block_size: i32,
}

impl Maze {
    pub fn load_from_file(path: &str, block_size: i32) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let grid: Vec<Vec<char>> = text.lines().map(|l| l.chars().collect()).collect();

        if grid.is_empty() || grid[0].is_empty() {
            return Err("maze vacío o inválido".into());
        }
        let height = grid.len();
        let width = grid[0].len();
        if !grid.iter().all(|row| row.len() == width) {
            return Err("todas las filas deben tener el mismo ancho".into());
        }

        // Validaciones mínimas
        let mut p = 0usize;
        let mut e = 0usize;
        for row in &grid {
            for &c in row {
                if c == 'P' {
                    p += 1;
                }
                if c == 'E' {
                    e += 1;
                }
            }
        }
        if p != 1 {
            return Err("debe haber exactamente 1 'P'".into());
        }
        if e == 0 {
            return Err("debe existir al menos una 'E'".into());
        }

        Ok(Self {
            grid,
            width,
            height,
            block_size,
        })
    }

    pub fn is_wall(&self, i: isize, j: isize) -> bool {
        if i < 0 || j < 0 {
            return true;
        }
        let (i, j) = (i as usize, j as usize);
        if j >= self.height || i >= self.width {
            return true;
        }
        let c = self.grid[j][i];
        c != ' ' && c != '.' && c != 'P' && c != 'E'
    }

    pub fn cell(&self, i: isize, j: isize) -> char {
        if i < 0 || j < 0 {
            return '#';
        }
        let (i, j) = (i as usize, j as usize);
        if j >= self.height || i >= self.width {
            return '#';
        }
        self.grid[j][i]
    }
}

fn color_for_cell(c: char) -> Color {
    match c {
        '#' => Color::DARKGRAY,
        'A' => Color::RED,
        'B' => Color::GREEN,
        'C' => Color::BLUE,
        'P' => Color::GOLD,
        'E' => Color::LIME,
        '.' | ' ' => Color::new(30, 30, 35, 255), // piso
        _ => Color::LIGHTGRAY,
    }
}

fn draw_cell(framebuffer: &mut FrameBuffer, x0: i32, y0: i32, size: i32, color: Color) {
    framebuffer.set_color(color);
    for y in y0..(y0 + size) {
        for x in x0..(x0 + size) {
            framebuffer.set_pixel(x, y);
        }
    }
}

pub fn render_maze_2d(framebuffer: &mut FrameBuffer, maze: &Maze) {
    // fondo
    framebuffer.set_color(Color::BLACK);
    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            framebuffer.set_pixel(x, y);
        }
    }

    // dibuja la grilla
    let bs = maze.block_size;
    for (j, row) in maze.grid.iter().enumerate() {
        for (i, &cell) in row.iter().enumerate() {
            let xo = (i as i32) * bs;
            let yo = (j as i32) * bs;
            draw_cell(framebuffer, xo, yo, bs, color_for_cell(cell));
        }
    }
}
