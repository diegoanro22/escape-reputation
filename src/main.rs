mod caster;
mod controller;
mod draw_utils;
mod framebuffer;
mod maze;
mod player;
mod render2d;
mod render3d;

use controller::process_input;
use framebuffer::FrameBuffer;
use maze::Maze;
use player::Player;
use render3d::render3d;

use raylib::prelude::*;

fn main() {
    let screen_w = 800;
    let screen_h = 600;

    let (mut rl, thread) = raylib::init()
        .size(screen_w, screen_h)
        .title("Swift Escape: El Laberinto")
        .build();

    // Oculta cursor para mouse-look suave (ESC lo vuelve a mostrar si cierras)
    rl.disable_cursor();

    let mut framebuffer = FrameBuffer::new(screen_w, screen_h, Color::BLACK);

    let mut mz = Maze::load_from_file("maze.txt", 48).expect("maze.txt no cargó");
    let mut p = Player::from_maze(
        &mut mz,
        std::f32::consts::FRAC_PI_3, // ángulo inicial
        std::f32::consts::FRAC_PI_3, // FOV ~60°
    );

    // Textura para “presentar” el framebuffer
    let mut tex = rl
        .load_texture_from_image(&thread, &framebuffer.color_buffer)
        .unwrap();

    rl.set_target_fps(60);

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        // 1) Input + movimiento con colisiones
        process_input(&rl, &mut p, &mz, dt);

        // 2) Render del mundo 3D en el FrameBuffer
        let _z = render3d(&mut framebuffer, &mz, &p);

        // 3) Subir el Image del framebuffer a la textura de la GPU
        tex = rl
            .load_texture_from_image(&thread, &framebuffer.color_buffer)
            .expect("no pude crear textura");

        // 4) Presentar en pantalla + HUD
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&tex, 0, 0, Color::WHITE);
        d.draw_fps(10, 10); // ✅ puntos de FPS del proyecto
        d.draw_text(
            "W/S adelante/atrás, A/D rotar, Q/E strafe, SHIFT correr",
            10,
            30,
            16,
            Color::RAYWHITE,
        );
        d.draw_text("Mouse: girar horizontal", 10, 52, 16, Color::RAYWHITE);
    }
}
