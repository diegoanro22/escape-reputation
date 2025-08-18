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
  LEYENDA DE TILES:
  '#' muro normal
  '.' piso
  'E' salida (paso al siguiente nivel)
  'P' personaje (spawn; se limpia a '.')
  'A' muro con textura 1
  'B' muro con textura 2
  'C' puerta (por ahora PASABLE)
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

        // Solo se permiten: # . E P A B C
        let mut p = 0usize;
        let mut e = 0usize;
        for row in &grid {
            for &c in row {
                match c {
                    '#' | '.' | 'E' | 'P' | 'A' | 'B' | 'C' => {}
                    _ => return Err(format!("símbolo no permitido: '{}'", c)),
                }
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

    // Bloquean: '#', 'A', 'B'.  Son piso/transitables: '.', 'E', 'C', 'P'
    #[inline]
    pub fn is_blocking(ch: char) -> bool {
        matches!(ch, '#' | 'A' | 'B')
    }
    #[inline]
    pub fn is_exit(ch: char) -> bool {
        ch == 'E'
    }
    #[inline]
    pub fn is_door(ch: char) -> bool {
        ch == 'C'
    }

    // Compat: algunos módulos antiguos consultaban "is_wall"
    #[inline]
    pub fn is_wall(&self, i: isize, j: isize) -> bool {
        Maze::is_blocking(self.cell(i, j))
    }

    pub fn cell_color(ch: char) -> Color {
        match ch {
            '.' => Color::new(30, 30, 35, 255), // piso
            '#' => Color::DARKGRAY,             // muro normal
            'A' => Color::BLUE,                 // muro textura 1
            'B' => Color::MAROON,               // muro textura 2
            'C' => Color::ORANGE,               // puerta
            'E' => Color::LIME,                 // salida
            'P' => Color::GOLD,                 // spawn
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
    // fondo
    framebuffer.set_color(Color::BLACK);
    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            framebuffer.set_pixel(x, y);
        }
    }

    // grilla
    let bs = maze.block_size;
    for (j, row) in maze.grid.iter().enumerate() {
        for (i, &cell) in row.iter().enumerate() {
            let xo = (i as i32) * bs;
            let yo = (j as i32) * bs;
            draw_cell(framebuffer, xo, yo, bs, Maze::cell_color(cell));
        }
    }
}
