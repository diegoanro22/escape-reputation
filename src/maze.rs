use crate::framebuffer::FrameBuffer;
use crate::player::Player;
use raylib::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    fs,
};

const AUTO_CLOSE_SECS: f32 = 1.5;

pub struct Maze {
    pub grid: Vec<Vec<char>>,
    pub width: usize,
    pub height: usize,
    pub block_size: i32,
    doors_open: HashSet<(usize, usize)>,       // puertas abiertas
    door_timers: HashMap<(usize, usize), f32>, // tiempo restante para autocierre
}

/*
  LEYENDA:
  '#' muro normal
  '.' piso
  'P' personaje (spawn; se limpia a '.')
  'A' muro con textura 1
  'B' muro con textura 2
  'C' puerta (cerrada = bloquea; abierta = NO bloquea ni se dibuja)
  'E' escaleras / salida (visible, NO bloquea)
  'F' final (visible, NO bloquea)
  'T' spawn del monstruo (no bloquea; visible en minimapa)
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

        let mut p = 0usize;
        let mut e = 0usize;
        for row in &grid {
            for &c in row {
                match c {
                    '#' | '.' | 'P' | 'A' | 'B' | 'C' | 'E' | 'F' | 'T' => {}
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
            doors_open: HashSet::new(),
            door_timers: HashMap::new(),
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

    // —— Estado de puertas ——
    #[inline]
    pub fn door_is_open(&self, i: usize, j: usize) -> bool {
        self.doors_open.contains(&(i, j))
    }

    pub fn toggle_door_at(&mut self, i: usize, j: usize) {
        if self.grid[j][i] != 'C' {
            return;
        }
        if !self.doors_open.remove(&(i, j)) {
            // abrir
            self.doors_open.insert((i, j));
            self.door_timers.insert((i, j), AUTO_CLOSE_SECS);
        } else {
            // cerrar manual
            self.door_timers.remove(&(i, j));
        }
    }

    // Puerta en frente usando un rayito corto (prioritario)
    fn toggle_door_in_front(&mut self, player: &Player, max_cells: f32) -> bool {
        let bs = self.block_size as f32;
        let mut d = 0.0_f32;
        let step = bs * 0.2;
        let maxd = max_cells * bs;

        while d <= maxd {
            let x = player.pos.x + player.a.cos() * d;
            let y = player.pos.y + player.a.sin() * d;
            if x < 0.0 || y < 0.0 || x >= (self.width as f32 * bs) || y >= (self.height as f32 * bs)
            {
                break;
            }

            let ci = (x / bs) as usize;
            let cj = (y / bs) as usize;
            let c = self.grid[cj][ci];
            if c == 'C' {
                self.toggle_door_at(ci, cj);
                return true;
            }
            // si hay muro duro enfrente, ya no hay puerta detrás
            if matches!(c, '#' | 'A' | 'B') {
                break;
            }

            d += step;
        }
        false
    }

    // Si no hay puerta enfrente, intentamos 4 adyacentes
    fn toggle_door_near(&mut self, player: &Player) -> bool {
        let bs = self.block_size as f32;
        let ci = (player.pos.x / bs) as i32;
        let cj = (player.pos.y / bs) as i32;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (i, j) = ((ci + dx) as isize, (cj + dy) as isize);
            if i < 0 || j < 0 {
                continue;
            }
            let (ui, uj) = (i as usize, j as usize);
            if uj >= self.height || ui >= self.width {
                continue;
            }
            if self.grid[uj][ui] == 'C' {
                self.toggle_door_at(ui, uj);
                return true;
            }
        }
        false
    }

    // Acción “usar”
    pub fn use_action(&mut self, player: &Player) -> bool {
        // prioridad: puerta enfrente (hasta ~1.5 celdas). Si no hay, adyacente.
        if self.toggle_door_in_front(player, 1.5) {
            return true;
        }
        self.toggle_door_near(player)
    }

    // —— Autocierre ——
    pub fn update_doors(&mut self, dt: f32) {
        let mut to_close: Vec<(usize, usize)> = Vec::new();
        for (k, t) in self.door_timers.iter_mut() {
            *t -= dt;
            if *t <= 0.0 {
                to_close.push(*k);
            }
        }
        for (i, j) in to_close {
            self.doors_open.remove(&(i, j));
            self.door_timers.remove(&(i, j));
        }
    }

    // —— Colisión ——
    #[inline]
    pub fn is_blocking_at(&self, i: isize, j: isize) -> bool {
        let c = self.cell(i, j);
        match c {
            '#' | 'A' | 'B' => true,
            'C' => {
                let (x, y) = (i as usize, j as usize);
                !self.door_is_open(x, y)
            }
            _ => false, // '.', 'E', 'F', 'P'
        }
    }

    // —— Superficie visible (para raycaster) ——
    #[inline]
    pub fn is_surface_at(&self, i: i32, j: i32) -> bool {
        let c: char = self.tile_at(i, j);
        match c {
            '#' | 'A' | 'B' => true,
            'C' => !self.door_is_open(i as usize, j as usize),
            'E' | 'F'  => true, // visibles, no bloquean
            _ => false,
        }
    }

    pub fn find_first(&self, tile: char) -> Option<(i32, i32)> {
        for (j, row) in self.grid.iter().enumerate() {
            for (i, &c) in row.iter().enumerate() {
                if c == tile {
                    return Some((i as i32, j as i32));
                }
            }
        }
        None
    }

    // —— 2D debug/minimapa (si lo usas) ——
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
            'T' => Color::PURPLE,
            _ => Color::LIGHTGRAY,
        }
    }
}

// helpers 2D (opcional)
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
