mod caster;
mod controller;
mod draw_utils;
mod enemy;
mod framebuffer;
mod levels;
mod maze;
mod player;
mod render3d;

use controller::process_input;
use draw_utils::draw_centered_text;
use enemy::Enemy;
use framebuffer::FrameBuffer;
use levels::{Levels, Transition};
use player::Player;
use raylib::prelude::*;
use render3d::render3d;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .title("Escape Reputation")
        .build();

    rl.disable_cursor();
    rl.set_target_fps(60);

    let mut framebuffer = FrameBuffer::new(800, 600, Color::BLACK);

    // Carga niveles
    let maps = vec![
        maze::Maze::load_from_file("levels/l0.txt", 48).expect("no l0"),
        maze::Maze::load_from_file("levels/l1.txt", 48).expect("no l1"),
        maze::Maze::load_from_file("levels/l2.txt", 48).expect("no l2"),
        maze::Maze::load_from_file("levels/l3.txt", 48).expect("no l3"),
        maze::Maze::load_from_file("levels/l4.txt", 48).expect("no l4"),
    ];
    let mut levels = Levels::new(maps);

    // Spawn
    let mut player = Player::from_maze(
        levels.active_mut(),
        std::f32::consts::FRAC_PI_3,
        std::f32::consts::FRAC_PI_3,
    );

    let mut won = false;
    let mut dead = false;
    let mut enemy: Option<Enemy> = None;

    let mut tex = rl
        .load_texture_from_image(&thread, &framebuffer.color_buffer)
        .unwrap();

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        if !won && !dead {
            {
                let maze = levels.active_mut();
                process_input(&rl, &mut player, maze, dt);
                maze.update_doors(dt);
            }
            if let Some(e) = enemy.as_mut() {
                if e.update(levels.active(), &player, dt) {
                    dead = true;
                }
            }

            // transiciones (E/F)
            match levels.check_transition(&player) {
                Transition::None => {}
                Transition::NextLevel => {
                    levels.advance_to_next(&mut player);
                    if levels.current >= 1 {
                        enemy = Some(Enemy::spawn_from_map_or_far(levels.active(), &player));
                    } else {
                        enemy = None;
                    }
                }
                Transition::Won => {
                    won = true;
                    enemy = None;
                }
            }
        }

        // render
        let mut z = render3d(&mut framebuffer, levels.active(), &player);

        if let Some(e) = &mut enemy {
            e.render_block3d(&mut framebuffer, levels.active(), &player, &mut z);
        }

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

        if won {
            draw_centered_text(&mut d, "¡GANASTE!", 220, 48, Color::BLACK);
            draw_centered_text(&mut d, "Presiona ESC para salir", 280, 20, Color::RAYWHITE);
        } else if dead {
            draw_centered_text(&mut d, "GAME OVER", 220, 48, Color::MAROON);
            draw_centered_text(&mut d, "Presiona ESC para salir", 280, 20, Color::RAYWHITE);
        }
    }
}
