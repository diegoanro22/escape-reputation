use crate::framebuffer::FrameBuffer;
use raylib::prelude::*;
use std::fs;

pub struct Maze {
    pub grid: Vec<Vec<char>>,
    pub width: usize,
    pub height: usize,
    pub block_size: i32,
}

/*
  LEYENDA:
  '#' muro normal
  '.' piso
  'P' personaje (spawn; se limpia a '.')
  'A' muro con textura 1
  'B' muro con textura 2
  'C' puerta (por ahora PASABLE)
  'E' escaleras / salida a siguiente nivel
  'F' salida FINAL (ganaste)
*/

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

        // Validación de símbolos
        let mut p = 0usize;
        let mut e = 0usize; // al menos una E o F
        for row in &grid {
            for &c in row {
                match c {
                    '#' | '.' | 'P' | 'A' | 'B' | 'C' | 'E' | 'F' => {}
                    _ => return Err(format!("símbolo no permitido: '{}'", c)),
                }
                if c == 'P' {
                    p += 1;
                }
                if c == 'E' || c == 'F' {
                    e += 1;
                }
            }
        }
        if p != 1 {
            return Err("debe haber exactamente 1 'P'".into());
        }
        if e == 0 {
            return Err("debe existir al menos una salida ('E' o 'F')".into());
        }

        Ok(Self {
            grid,
            width,
            height,
            block_size,
        })
    }

    #[inline]
    pub fn cell(&self, i: isize, j: isize) -> char {
        if i < 0 || j < 0 {
            return '#';
        }
        let (x, y) = (i as usize, j as usize);
        if y >= self.height || x >= self.width {
            return '#';
        }
        self.grid[y][x]
    }

    #[inline]
    pub fn tile_at(&self, i: i32, j: i32) -> char {
        self.cell(i as isize, j as isize)
    }

    // Bloquean: '#', 'A', 'B'
    #[inline]
    pub fn is_blocking(ch: char) -> bool {
        matches!(ch, '#' | 'A' | 'B')
    }

    // Triggers
    #[inline]
    pub fn is_exit_next(ch: char) -> bool {
        ch == 'E'
    }
    #[inline]
    pub fn is_exit_final(ch: char) -> bool {
        ch == 'F'
    }
    #[inline]
    pub fn is_door(ch: char) -> bool {
        ch == 'C'
    }

    // Colores 2D (minimapa)
    pub fn cell_color(ch: char) -> Color {
        match ch {
            '.' => Color::new(30, 30, 35, 255),
            '#' => Color::DARKGRAY,
            'A' => Color::BLUE,
            'B' => Color::MAROON,
            'C' => Color::ORANGE,
            'E' => Color::LIME,
            'F' => Color::GOLD,
            'P' => Color::GOLD,
            _ => Color::LIGHTGRAY,
        }
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
    framebuffer.set_color(Color::BLACK);
    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            framebuffer.set_pixel(x, y);
        }
    }
    let bs = maze.block_size;
    for (j, row) in maze.grid.iter().enumerate() {
        for (i, &cell) in row.iter().enumerate() {
            let xo = (i as i32) * bs;
            let yo = (j as i32) * bs;
            draw_cell(framebuffer, xo, yo, bs, Maze::cell_color(cell));
        }
    }
}
