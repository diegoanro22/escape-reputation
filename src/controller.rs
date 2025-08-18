use crate::{maze::Maze, player::Player};
use raylib::prelude::*;

const PLAYER_RADIUS: f32 = 10.0;
const MOUSE_SENS: f32 = 0.0035;

fn collides(maze: &Maze, x: f32, y: f32, r: f32) -> bool {
    let bs = maze.block_size as f32;
    let tests = [
        (x - r, y - r),
        (x + r, y - r),
        (x - r, y + r),
        (x + r, y + r),
    ];
    for (tx, ty) in tests {
        let i = (tx / bs) as i32;
        let j = (ty / bs) as i32;
        if Maze::is_blocking(maze.tile_at(i, j)) {
            return true;
        }
    }
    false
}

fn try_move(player: &mut Player, maze: &Maze, dx: f32, dy: f32) {
    let nx = player.pos.x + dx;
    let ny = player.pos.y + dy;

    if !collides(maze, nx, player.pos.y, PLAYER_RADIUS) {
        player.pos.x = nx;
    }
    if !collides(maze, player.pos.x, ny, PLAYER_RADIUS) {
        player.pos.y = ny;
    }
}

fn normalize_angle(a: f32) -> f32 {
    let mut a = a;
    while a <= -std::f32::consts::PI {
        a += std::f32::consts::TAU;
    }
    while a > std::f32::consts::PI {
        a -= std::f32::consts::TAU;
    }
    a
}

pub fn process_input(rl: &RaylibHandle, player: &mut Player, maze: &Maze, dt: f32) {
    // Rotación por mouse (horizontal)
    let md = rl.get_mouse_delta();
    player.a += (md.x) * MOUSE_SENS;
    player.a = normalize_angle(player.a);

    // Movimiento
    let mut dir = 0.0;
    if rl.is_key_down(KeyboardKey::KEY_UP) || rl.is_key_down(KeyboardKey::KEY_W) {
        dir += 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S) {
        dir -= 1.0;
    }

    // Strafe
    let mut strafe = 0.0;
    if rl.is_key_down(KeyboardKey::KEY_A) || rl.is_key_down(KeyboardKey::KEY_LEFT) {
        strafe -= 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) || rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        strafe += 1.0;
    }

    let speed = if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
        player.move_speed * 1.8
    } else {
        player.move_speed
    };

    let forward_dx = player.a.cos() * speed * dir * dt;
    let forward_dy = player.a.sin() * speed * dir * dt;
    let strafe_dx = (-player.a.sin()) * speed * strafe * dt;
    let strafe_dy = (player.a.cos()) * speed * strafe * dt;

    try_move(player, maze, forward_dx + strafe_dx, forward_dy + strafe_dy);
}
