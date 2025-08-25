use crate::{maze::Maze, player::Player};
use raylib::prelude::*;
// Import explícito por si el prelude de tu versión no lo reexporta:
use raylib::prelude::{GamepadAxis, GamepadButton};

const PLAYER_RADIUS: f32 = 10.0;
const MOUSE_SENS: f32 = 0.0035;

// ======= Parámetros de gamepad =======
const GAMEPAD_ID: i32 = 0;            // primer control
const STICK_DEADZONE: f32 = 0.18;     // zona muerta para sticks
const PAD_YAW_SENS: f32 = 2.6;        // rad/seg de giro a deflexión 1.0
const PAD_RUN_THRESHOLD: f32 = 0.55;  // umbral de LT/RT para sprint
// =====================================

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

fn normalize_angle(mut a: f32) -> f32 {
    while a <= -std::f32::consts::PI {
        a += std::f32::consts::TAU;
    }
    while a > std::f32::consts::PI {
        a -= std::f32::consts::TAU;
    }
    a
}

// Zona muerta + reescalado (curva lineal)
#[inline]
fn dz(x: f32, dead: f32) -> f32 {
    if x.abs() < dead { 0.0 } else { ((x.abs() - dead) / (1.0 - dead)) * x.signum() }
}

/// Devuelve `true` si se abrió/cerró alguna puerta este frame.
pub fn process_input(rl: &RaylibHandle, player: &mut Player, maze: &mut Maze, dt: f32) -> bool {
    // -------- Rotación (mouse + stick derecho) --------
    let md = rl.get_mouse_delta();
    let mut yaw_delta = md.x * MOUSE_SENS;

    if rl.is_gamepad_available(GAMEPAD_ID) {
        // Giro con stick derecho (eje X)
        let rx = rl.get_gamepad_axis_movement(GAMEPAD_ID, GamepadAxis::GAMEPAD_AXIS_RIGHT_X);
    }
    player.a = normalize_angle(player.a + yaw_delta);

    // -------- Movimiento (teclado + stick izquierdo) --------
    let mut dir = 0.0;
    if rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP)   { dir += 1.0; }
    if rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_DOWN) { dir -= 1.0; }

    let mut strafe = 0.0;
    if rl.is_key_down(KeyboardKey::KEY_A) || rl.is_key_down(KeyboardKey::KEY_LEFT)  { strafe -= 1.0; }
    if rl.is_key_down(KeyboardKey::KEY_D) || rl.is_key_down(KeyboardKey::KEY_RIGHT) { strafe += 1.0; }

    if rl.is_gamepad_available(GAMEPAD_ID) {
        let lx = rl.get_gamepad_axis_movement(GAMEPAD_ID, GamepadAxis::GAMEPAD_AXIS_LEFT_X);
        let ly = rl.get_gamepad_axis_movement(GAMEPAD_ID, GamepadAxis::GAMEPAD_AXIS_LEFT_Y);
        // En raylib: Y arriba ≈ -1. Queremos "adelante = +1".
        strafe += dz(lx, STICK_DEADZONE);
        dir    += -dz(ly, STICK_DEADZONE);
    }

    // -------- Sprint (Shift o gatillos LT/RT) --------
    let mut sprint = rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT);
    if rl.is_gamepad_available(GAMEPAD_ID) {
        let lt = rl.get_gamepad_axis_movement(GAMEPAD_ID, GamepadAxis::GAMEPAD_AXIS_LEFT_TRIGGER);
        let rt = rl.get_gamepad_axis_movement(GAMEPAD_ID, GamepadAxis::GAMEPAD_AXIS_RIGHT_TRIGGER);
        if lt > PAD_RUN_THRESHOLD || rt > PAD_RUN_THRESHOLD {
            sprint = true;
        }
    }

    let speed = if sprint { player.move_speed * 1.8 } else { player.move_speed };

    let forward_dx = player.a.cos() * speed * dir * dt;
    let forward_dy = player.a.sin() * speed * dir * dt;
    let strafe_dx  = (-player.a.sin()) * speed * strafe * dt;
    let strafe_dy  = ( player.a.cos()) * speed * strafe * dt;

    try_move(player, maze, forward_dx + strafe_dx, forward_dy + strafe_dy);

    // -------- Usar puerta (E o botón A del gamepad) --------
    let mut toggled = false;
    if rl.is_key_pressed(KeyboardKey::KEY_E) {
        toggled = maze.use_action(player);
    }
    if rl.is_gamepad_available(GAMEPAD_ID) {
        // A / Cross equivale a "usar"
        if rl.is_gamepad_button_pressed(GAMEPAD_ID, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN) {
            toggled = maze.use_action(player) || toggled;
        }
    }

    toggled
}
