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
use levels::Levels;
use player::Player;
use render3d::{draw_exit_markers, render3d};

use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .title("Swift Escape: El Laberinto")
        .build();

    rl.disable_cursor();

    let mut framebuffer = FrameBuffer::new(800, 600, Color::BLACK);

    // Carga tus niveles (mismo tamaño/block_size recomendado)
    let maps = vec![
        maze::Maze::load_from_file("levels/l0.txt", 48).expect("no l0"),
        maze::Maze::load_from_file("levels/l1.txt", 48).expect("no l1"),
        // agrega más: l2, l3, ...
    ];
    let mut levels = Levels::new(maps);

    // spawnea jugador en el 'P' del nivel 0
    let mut player = Player::from_maze(
        levels.active_mut(),
        std::f32::consts::FRAC_PI_3,
        std::f32::consts::FRAC_PI_3,
    );

    let mut tex = rl
        .load_texture_from_image(&thread, &framebuffer.color_buffer)
        .unwrap();

    rl.set_target_fps(60);
    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        // input + movimiento con colisiones contra el maze ACTIVO
        process_input(&rl, &mut player, levels.active(), dt);

        if levels.try_advance_on_exit(&mut player) {
            println!("→ Nivel {}!", levels.current);
        }

        // render 3D del maze ACTIVO
        let zbuffer = render3d(&mut framebuffer, levels.active(), &player);
        draw_exit_markers(&mut framebuffer, levels.active(), &player, &zbuffer);

        // presentar
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
    }
}
