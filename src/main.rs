mod audio;
mod caster;
mod controller;
mod draw_utils;
mod enemy;
mod framebuffer;
mod levels;
mod maze;
mod player;
mod render3d;
mod textures;

use audio::AudioAssets;
use controller::process_input;
use draw_utils::draw_centered_text;
use enemy::Enemy;
use framebuffer::FrameBuffer;
use levels::{Levels, Transition};
use player::Player;
use raylib::core::audio::RaylibAudio;
use raylib::prelude::*;
use render3d::render3d;
use textures::Textures;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .title("Escape Reputation")
        .build();

    rl.disable_cursor();
    rl.set_target_fps(60);

    // === AUDIO ===
    let audio = RaylibAudio::init_audio_device().expect("audio");
    audio.set_master_volume(0.85);
    let mut sfx = AudioAssets::new(&audio).expect("load audio");

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

    let textures = Textures::load_default();

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
                    // golpe del enemigo al matarte
                    // sfx.sfx_hit(1.0);
                    dead = true;
                }
            }

            // transiciones (E/F)
            match levels.check_transition(&player) {
                Transition::None => {}
                Transition::NextLevel => {
                    levels.advance_to_next(&mut player);
                    // (opcional) sonido de “puerta/paso de nivel”
                    // sfx.sfx_door(0.8);

                    if levels.current >= 1 {
                        enemy = Some(Enemy::spawn_from_map_or_far(levels.active(), &player));
                    } else {
                        enemy = None;
                    }
                }
                Transition::Won => {
                    won = true;
                    enemy = None;
                    // 🔉 baja música al ganar
                    sfx.set_music_volume(0.15);
                }
            }
        }

        // === Audio: SIEMPRE actualiza el stream ===
        // Si hay enemigo y sigues vivo, usa su posición; si no, None (música bajita)
        let enemy_pos = if !won && !dead {
            enemy.as_ref().map(|e| e.pos)
        } else {
            None
        };
        sfx.update(dt, player.pos, enemy_pos);

        if dead {
            // baja música fuerte cuando mueres (o podrías hacer sfx.pause_music())
            sfx.set_music_volume(0.0);
        }

        // ---- render igualito ----
        let mut z = render3d(&mut framebuffer, levels.active(), &player, &textures);
        if let Some(e) = &mut enemy {
            e.render_sprite3d(
                &mut framebuffer,
                levels.active(),
                &player,
                &mut z,
                &textures,
            );
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
