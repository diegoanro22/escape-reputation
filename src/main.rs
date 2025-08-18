mod caster;
mod controller;
mod draw_utils;
mod framebuffer;
mod levels;
mod maze;
mod player;
mod render3d;

use controller::process_input;
use framebuffer::FrameBuffer;
use levels::{Levels, Transition};
use player::Player;
use raylib::prelude::*;
use render3d::{draw_markers_as_blocks, render3d};

fn draw_centered_text(d: &mut RaylibDrawHandle, text: &str, y: i32, size: i32, color: Color) {
    let w = d.get_screen_width();
    let tw = d.measure_text(text, size);
    d.draw_text(text, (w - tw) / 2, y, size, color);
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .title("Escape Reputation")
        .build();

    rl.disable_cursor();
    rl.set_target_fps(60);

    let mut framebuffer = FrameBuffer::new(800, 600, Color::BLACK);

    let maps = vec![
        maze::Maze::load_from_file("levels/l0.txt", 48).expect("no l0"),
        maze::Maze::load_from_file("levels/l1.txt", 48).expect("no l1"),
        // Puedes agregar un l_final.txt con 'F' adentro para el cierre
    ];
    let mut levels = Levels::new(maps);

    let mut player = Player::from_maze(
        levels.active_mut(),
        std::f32::consts::FRAC_PI_3,
        std::f32::consts::FRAC_PI_3,
    );

    let mut won = false; // estado simple de victoria
    let mut tex = rl
        .load_texture_from_image(&thread, &framebuffer.color_buffer)
        .unwrap();

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        if !won {
            process_input(&rl, &mut player, levels.active(), dt);
            match levels.check_transition(&player) {
                Transition::None => {}
                Transition::NextLevel => levels.advance_to_next(&mut player),
                Transition::Won => {
                    won = true;
                }
            }
        }

        let zbuffer = render3d(&mut framebuffer, levels.active(), &player);
        draw_markers_as_blocks(&mut framebuffer, levels.active(), &player, &zbuffer);

        tex = rl
            .load_texture_from_image(&thread, &framebuffer.color_buffer)
            .unwrap();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&tex, 0, 0, Color::WHITE);
        d.draw_fps(10, 10);
        d.draw_text(
            &format!("Nivel: {}", levels.current),
            10,
            30,
            16,
            Color::RAYWHITE,
        );

        if won {
            // HUD “GANASTE” (luego aquí llamas a tu cinemática)
            draw_centered_text(&mut d, "¡GANASTE!", 220, 48, Color::GOLD);
            draw_centered_text(&mut d, "Presiona ESC para salir", 280, 20, Color::RAYWHITE);
        }
    }
}
