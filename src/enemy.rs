use crate::{framebuffer::FrameBuffer, maze::Maze, player::Player};
use raylib::prelude::*;
use std::collections::{HashMap, VecDeque};

const ENEMY_RADIUS: f32 = 10.0;
const ENEMY_SPEED: f32 = 95.0;
const ENEMY_CHASE_SPEED: f32 = 125.0;
const KILL_DIST: f32 = 16.0;
const REPATH_EVERY: f32 = 0.6;
const REACH_WAYPOINT: f32 = 6.0;
const MIN_SPAWN_DIST_CELLS: i32 = 10;
const SPAWN_GRACE_SECS: f32 = 1.2;
const RETREAT_UNREACHABLE_SECS: f32 = 2.5;

// tamaño del bloque (fracción del block_size) para el AABB del viejo render
const BOX_HALF: f32 = 0.35;

pub struct Enemy {
    pub pos: Vector2,      // píxeles
    path: Vec<(i32, i32)>, // ruta (celdas)
    time_to_repath: f32,
    awake: f32,
    frustration: f32,
}

impl Enemy {
    /// Spawnea en 'T' si existe y está >= MIN_SPAWN_DIST_CELLS del jugador.
    /// Si no, usa la celda caminable más lejana.
    pub fn spawn_from_map_or_far(maze: &Maze, player: &Player) -> Self {
        let bs = maze.block_size as f32;
        let start = Self::cell_of(player.pos, bs);
        let distmap = bfs_distances(maze, start);

        if let Some((ti, tj)) = maze.find_first('T') {
            let d = distmap.get(&(ti, tj)).copied().unwrap_or(-1);
            if d >= MIN_SPAWN_DIST_CELLS {
                return Self {
                    pos: Vector2::new((ti as f32 + 0.5) * bs, (tj as f32 + 0.5) * bs),
                    path: Vec::new(),
                    time_to_repath: 0.0,
                    awake: SPAWN_GRACE_SECS,
                    frustration: 0.0,
                };
            }
        }
        // fallback: más lejano posible
        let mut best: Option<((i32, i32), i32)> = None;
        for j in 0..maze.height as i32 {
            for i in 0..maze.width as i32 {
                if !is_walkable_cell(maze, i, j) {
                    continue;
                }
                if let Some(d) = distmap.get(&(i, j)) {
                    if best.map_or(true, |(_, bd)| *d > bd) {
                        best = Some(((i, j), *d));
                    }
                }
            }
        }
        let spawn_cell = best.map(|(c, _)| c).unwrap_or(start);
        let spawn = Vector2::new(
            (spawn_cell.0 as f32 + 0.5) * bs,
            (spawn_cell.1 as f32 + 0.5) * bs,
        );

        Self {
            pos: spawn,
            path: Vec::new(),
            time_to_repath: 0.0,
            awake: SPAWN_GRACE_SECS,
            frustration: 0.0,
        }
    }

    fn retreat_far_from_player(&mut self, maze: &Maze, player: &Player) {
        let bs = maze.block_size as f32;
        let player_cell = Self::cell_of(player.pos, bs);
        let dist_from_player = bfs_distances(maze, player_cell);

        let mut best: Option<((i32, i32), i32)> = None;
        for j in 0..maze.height as i32 {
            for i in 0..maze.width as i32 {
                if !is_walkable_cell(maze, i, j) {
                    continue;
                }
                let score = dist_from_player.get(&(i, j)).copied().unwrap_or(99_999);
                if best.map_or(true, |(_, bd)| score > bd) {
                    best = Some(((i, j), score));
                }
            }
        }
        if let Some(((ci, cj), _)) = best {
            self.pos = Vector2::new((ci as f32 + 0.5) * bs, (cj as f32 + 0.5) * bs);
        }
        self.path.clear();
        self.time_to_repath = 0.0;
        self.awake = SPAWN_GRACE_SECS;
        self.frustration = 0.0;
    }

    pub fn update(&mut self, maze: &Maze, player: &Player, dt: f32) -> bool {
        if self.awake > 0.0 {
            self.awake -= dt;
            return false;
        }
        if self.pos.distance_to(player.pos) <= (KILL_DIST + ENEMY_RADIUS) {
            return true;
        }

        let bs = maze.block_size as f32;
        let chase = has_los(maze, self.pos, player.pos);
        let spd = if chase {
            ENEMY_CHASE_SPEED
        } else {
            ENEMY_SPEED
        };
        let mut target = player.pos;

        if !chase {
            self.time_to_repath -= dt;
            if self.path.is_empty() || self.time_to_repath <= 0.0 {
                let from = Self::cell_of(self.pos, bs);
                let to = Self::cell_of(player.pos, bs);
                self.path = bfs_path(maze, from, to);
                self.time_to_repath = REPATH_EVERY;
            }
            if self.path.is_empty() {
                self.frustration += dt;
                if self.frustration >= RETREAT_UNREACHABLE_SECS {
                    self.retreat_far_from_player(maze, player);
                    return false;
                }
            } else {
                self.frustration = 0.0;
                if let Some(&(ci, cj)) = self.path.first() {
                    let waypoint = Vector2::new((ci as f32 + 0.5) * bs, (cj as f32 + 0.5) * bs);
                    if self.pos.distance_to(waypoint) <= REACH_WAYPOINT {
                        let _ = self.path.remove(0);
                        if let Some(&(ni, nj)) = self.path.first() {
                            target = Vector2::new((ni as f32 + 0.5) * bs, (nj as f32 + 0.5) * bs);
                        }
                    } else {
                        target = waypoint;
                    }
                }
            }
        } else {
            self.frustration = 0.0;
        }

        let (dx, dy) = dir_towards(self.pos, target);
        let step = spd * dt;
        try_move_enemy(self, maze, dx * step, dy * step);

        self.pos.distance_to(player.pos) <= (KILL_DIST + ENEMY_RADIUS)
    }

    /// ===== Sprite billboard con textura (PNG con alpha) =====
    pub fn render_sprite3d(
        &self,
        framebuffer: &mut FrameBuffer,
        maze: &Maze,
        player: &Player,
        zbuffer: &mut [f32],
        textures: &crate::textures::Textures,
    ) {
        let w = framebuffer.width as i32;
        let h = framebuffer.height as i32;
        let hw = w as f32 * 0.5;
        let hh = h as f32 * 0.5;
        let dist_to_proj = hw / (player.fov * 0.5).tan();
        let bs = maze.block_size as f32;

        // dirección adelante y derecha de la cámara
        let dirx = player.a.cos();
        let diry = player.a.sin();
        let rightx = -diry;
        let righty = dirx;
        let plane_len = (player.fov * 0.5).tan(); // escala del “plano de cámara”

        // vector desde cámara a enemigo
        let vx = self.pos.x - player.pos.x;
        let vy = self.pos.y - player.pos.y;

        // componente hacia adelante (perpendicular a pantalla)
        let perp = vx * dirx + vy * diry;
        if perp <= 1.0 {
            return;
        } // detrás o demasiado cerca

        // componente lateral (para x en pantalla)
        let side = vx * rightx + vy * righty;
        let screen_x = hw * (1.0 + (side / (perp * plane_len)));

        // altura del sprite en pantalla (≈ tamaño de un bloque)
        let sprite_h = (bs * dist_to_proj) / perp;
        let tex = textures.get('M');
        let sprite_w = sprite_h * (tex.w as f32 / tex.h as f32);

        let mut x0 = (screen_x - sprite_w * 0.5).floor() as i32;
        let mut x1 = (screen_x + sprite_w * 0.5).ceil() as i32;
        let mut y0 = (hh - sprite_h * 0.5).floor() as i32;
        let mut y1 = (hh + sprite_h * 0.5).ceil() as i32;

        // recorta a pantalla
        x0 = x0.clamp(0, w - 1);
        x1 = x1.clamp(0, w - 1);
        y0 = y0.clamp(0, h - 1);
        y1 = y1.clamp(0, h - 1);
        if x0 > x1 || y0 > y1 {
            return;
        }

        let u_eps = 0.5 / tex.w as f32;
        let v_eps = 0.5 / tex.h as f32;

        // niebla leve
        let sky = Color::new(20, 24, 40, 255);
        let fog_t = 1.0 - (-perp * 0.010).exp();

        for sx in x0..=x1 {
            // oclusión con paredes
            let col = sx as usize;
            if col < zbuffer.len() && perp >= zbuffer[col] {
                continue;
            }

            let u =
                ((sx as f32 - (screen_x - sprite_w * 0.5)) / sprite_w).clamp(u_eps, 1.0 - u_eps);

            for sy in y0..=y1 {
                let v = ((sy as f32 - (hh - sprite_h * 0.5)) / sprite_h).clamp(v_eps, 1.0 - v_eps);

                let c = tex.sample(u, v);
                if c.a < 16 {
                    continue;
                } // respeta transparencia

                let mut out = c;
                // mezcla con “niebla” para profundidad
                out.r = (out.r as f32 * (1.0 - fog_t) + sky.r as f32 * fog_t) as u8;
                out.g = (out.g as f32 * (1.0 - fog_t) + sky.g as f32 * fog_t) as u8;
                out.b = (out.b as f32 * (1.0 - fog_t) + sky.b as f32 * fog_t) as u8;

                framebuffer.set_color(out);
                framebuffer.set_pixel(sx, sy);
            }
            // si quieres que el sprite tape a otros sprites detrás:
            zbuffer[col] = perp;
        }
    }

    // --- utilidades ---

    #[allow(dead_code)]
    pub fn render_block3d(
        &self,
        framebuffer: &mut FrameBuffer,
        maze: &Maze,
        player: &Player,
        zbuffer: &mut [f32],
    ) {
        // (tu render morado anterior por si quieres debug; lo puedes dejar o borrar)
        let w = framebuffer.width as i32;
        let hw = framebuffer.width as f32 * 0.5;
        let hh = framebuffer.height as f32 * 0.5;
        let dist_to_proj = hw / (player.fov * 0.5).tan();
        let hs = maze.block_size as f32 * BOX_HALF;
        let minx = self.pos.x - hs;
        let maxx = self.pos.x + hs;
        let miny = self.pos.y - hs;
        let maxy = self.pos.y + hs;
        let base = Color::new(180, 40, 220, 255);

        for sx in 0..w {
            let t = sx as f32 / (w as f32);
            let ray_angle = player.a - (player.fov * 0.5) + (player.fov * t);
            let dir = Vector2::new(ray_angle.cos(), ray_angle.sin());
            if let Some(t_hit) = ray_aabb_2d(player.pos, dir, minx, maxx, miny, maxy) {
                let delta = (ray_angle - player.a).cos().abs().max(1e-6);
                let dist_perp = t_hit * delta;
                let col = sx as usize;
                if col < zbuffer.len() && dist_perp >= zbuffer[col] {
                    continue;
                }
                let stake_h = (maze.block_size as f32 * dist_to_proj) / dist_perp;
                let top = ((hh - stake_h * 0.5).max(0.0)) as i32;
                let bot = ((hh + stake_h * 0.5).min(framebuffer.height as f32 - 1.0)) as i32;
                let fade = (1.0 / (1.0 + dist_perp * 0.002)).clamp(0.25, 1.0);
                let c = Color::new(
                    (base.r as f32 * fade) as u8,
                    (base.g as f32 * fade) as u8,
                    (base.b as f32 * fade) as u8,
                    255,
                );
                framebuffer.set_color(c);
                for y in top..=bot {
                    framebuffer.set_pixel(sx, y);
                }
                zbuffer[col] = dist_perp;
            }
        }
    }

    #[inline]
    fn cell_of(pos: Vector2, bs: f32) -> (i32, i32) {
        ((pos.x / bs) as i32, (pos.y / bs) as i32)
    }
}

// ---- intersección rayo–AABB 2D ----
fn ray_aabb_2d(
    origin: Vector2,
    dir: Vector2,
    minx: f32,
    maxx: f32,
    miny: f32,
    maxy: f32,
) -> Option<f32> {
    let invx = if dir.x.abs() < 1e-6 {
        f32::INFINITY
    } else {
        1.0 / dir.x
    };
    let invy = if dir.y.abs() < 1e-6 {
        f32::INFINITY
    } else {
        1.0 / dir.y
    };
    let (mut tx1, mut tx2) = ((minx - origin.x) * invx, (maxx - origin.x) * invx);
    if tx1 > tx2 {
        std::mem::swap(&mut tx1, &mut tx2);
    }
    let (mut ty1, mut ty2) = ((miny - origin.y) * invy, (maxy - origin.y) * invy);
    if ty1 > ty2 {
        std::mem::swap(&mut ty1, &mut ty2);
    }
    let t_enter = tx1.max(ty1);
    let t_exit = tx2.min(ty2);
    if t_exit < 0.0 || t_enter > t_exit {
        return None;
    }
    let t_hit = if t_enter >= 0.0 {
        t_enter
    } else {
        t_exit.max(0.0)
    };
    if t_hit.is_finite() { Some(t_hit) } else { None }
}


/* -------------------- IA / movimiento / pathfinding -------------------- */

fn dir_towards(from: Vector2, to: Vector2) -> (f32, f32) {
    let mut v = to - from;
    let len = (v.x * v.x + v.y * v.y).sqrt();
    if len > 1e-5 {
        v /= len;
    }
    (v.x, v.y)
}

fn try_move_enemy(en: &mut Enemy, maze: &Maze, dx: f32, dy: f32) {
    let nx = en.pos.x + dx;
    let ny = en.pos.y + dy;
    if !collides(maze, nx, en.pos.y, ENEMY_RADIUS) {
        en.pos.x = nx;
    }
    if !collides(maze, en.pos.x, ny, ENEMY_RADIUS) {
        en.pos.y = ny;
    }
}

fn collides(maze: &Maze, x: f32, y: f32, r: f32) -> bool {
    let bs = maze.block_size as f32;
    let tests = [
        (x - r, y - r),
        (x + r, y - r),
        (x - r, y + r),
        (x + r, y + r),
    ];
    for (tx, ty) in tests {
        let i = (tx / bs) as isize;
        let j = (ty / bs) as isize;
        if maze.is_blocking_at(i, j) {
            return true;
        }
    }
    false
}

fn has_los(maze: &Maze, from: Vector2, to: Vector2) -> bool {
    let bs = maze.block_size as f32;
    let mut d = 0.0f32;
    let total = from.distance_to(to);
    if total < 1.0 {
        return true;
    }
    let step = (bs * 0.25).max(2.0);
    let dir = (to - from) / total;

    while d <= total {
        let p = from + dir * d;
        let ci = (p.x / bs) as isize;
        let cj = (p.y / bs) as isize;
        if maze.is_blocking_at(ci, cj) {
            return false;
        }
        d += step;
    }
    true
}

fn is_walkable_cell(maze: &Maze, i: i32, j: i32) -> bool {
    if i < 0 || j < 0 {
        return false;
    }
    let c = maze.tile_at(i, j);
    match c {
        '#' | 'A' | 'B' => false,
        'C' => maze.door_is_open(i as usize, j as usize),
        'T' => true,
        _ => true, // '.', 'E', 'F', 'P'
    }
}

fn center_of_cell(cell: (i32, i32), bs: f32) -> Vector2 {
    Vector2::new((cell.0 as f32 + 0.5) * bs, (cell.1 as f32 + 0.5) * bs)
}

fn bfs_distances(maze: &Maze, start: (i32, i32)) -> HashMap<(i32, i32), i32> {
    let mut dist: HashMap<(i32, i32), i32> = HashMap::new();
    let mut q = VecDeque::new();
    q.push_back(start);
    dist.insert(start, 0);

    while let Some((i, j)) = q.pop_front() {
        let d = dist[&(i, j)];
        for (ni, nj) in [(i + 1, j), (i - 1, j), (i, j + 1), (i, j - 1)] {
            if ni < 0 || nj < 0 {
                continue;
            }
            if ni as usize >= maze.width || nj as usize >= maze.height {
                continue;
            }
            if dist.contains_key(&(ni, nj)) {
                continue;
            }
            if !is_walkable_cell(maze, ni, nj) {
                continue;
            }
            dist.insert((ni, nj), d + 1);
            q.push_back((ni, nj));
        }
    }
    dist
}

fn bfs_path(maze: &Maze, start: (i32, i32), goal: (i32, i32)) -> Vec<(i32, i32)> {
    if start == goal {
        return Vec::new();
    }
    let mut prev: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut q = VecDeque::new();
    let mut seen: HashMap<(i32, i32), bool> = HashMap::new();

    q.push_back(start);
    seen.insert(start, true);

    while let Some((i, j)) = q.pop_front() {
        for (ni, nj) in [(i + 1, j), (i - 1, j), (i, j + 1), (i, j - 1)] {
            if ni < 0 || nj < 0 {
                continue;
            }
            if ni as usize >= maze.width || nj as usize >= maze.height {
                continue;
            }
            if seen.get(&(ni, nj)).copied().unwrap_or(false) {
                continue;
            }
            if !is_walkable_cell(maze, ni, nj) {
                continue;
            }

            prev.insert((ni, nj), (i, j));
            seen.insert((ni, nj), true);
            if (ni, nj) == goal {
                let mut path = Vec::new();
                let mut cur = (ni, nj);
                while cur != start {
                    path.push(cur);
                    cur = prev[&cur];
                }
                path.reverse();
                return path;
            }
            q.push_back((ni, nj));
        }
    }
    Vec::new()
}
