mod audio;
mod caster;
mod controller;
mod draw_utils;
mod enemy;
mod framebuffer;
mod levels;
mod maze;
mod menu;
mod player;
mod render3d;
mod textures;

use audio::AudioAssets;
use controller::process_input;
use draw_utils::draw_centered_text;
use enemy::Enemy;
use framebuffer::FrameBuffer;
use levels::{Levels, Transition};
use menu::{Menu, MenuOutcome};
use player::Player;
use raylib::core::audio::RaylibAudio;
use raylib::prelude::*;
use render3d::render3d;
use textures::Textures;

enum AppState {
    Menu,
    Playing,
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .title("Escape Reputation")
        .build();

    rl.set_exit_key(Some(KeyboardKey::KEY_ESCAPE));
    rl.enable_cursor();

    // Flags para cambiar el modo del cursor después del dibujo
    let mut want_enter_play: bool = false;
    let mut want_back_to_menu: bool = false;

    rl.set_target_fps(60);

    // === AUDIO ===
    let audio = RaylibAudio::init_audio_device().expect("audio");
    audio.set_master_volume(0.85);
    let mut sfx = AudioAssets::new(&audio).expect("load audio");

    // Framebuffer para el render 3D
    let mut framebuffer = FrameBuffer::new(800, 600, Color::BLACK);
    let mut tex = rl
        .load_texture_from_image(&thread, &framebuffer.color_buffer)
        .unwrap();

    // Carga niveles
    let maps = vec![
        maze::Maze::load_from_file("levels/l0.txt", 48).expect("no l0"),
        maze::Maze::load_from_file("levels/l1.txt", 48).expect("no l1"),
        maze::Maze::load_from_file("levels/l2.txt", 48).expect("no l2"),
        maze::Maze::load_from_file("levels/l3.txt", 48).expect("no l3"),
        maze::Maze::load_from_file("levels/l4.txt", 48).expect("no l4"),
    ];
    let mut levels = Levels::new(maps);
    let total_levels = levels.total_levels();

    // Spawn inicial (usa un mapa temporal para no borrar 'P' en levels)
    let mut tmp = maze::Maze::load_from_file("levels/l0.txt", 48).expect("no l0 tmp");
    let mut player = Player::from_maze(
        &mut tmp,
        std::f32::consts::FRAC_PI_3,
        std::f32::consts::FRAC_PI_3,
    );

    // Progreso
    let mut unlocked: usize = 1;

    // Menú
    let mut menu = Menu::new(&mut rl, &thread, total_levels);
    menu.set_unlocked(unlocked);

    // Estado de juego y enemigos
    let mut state = AppState::Menu;
    let textures = Textures::load_default();
    let game_over_tex: Option<Texture2D> = rl.load_texture(&thread, "assets/game_over.png").ok();
    let mut enemy: Option<Enemy> = None;
    let mut won = false;
    let mut dead = false;

    // SFX pasos
    let mut last_player_pos = player.pos;
    let mut step_accum = 0.0f32;
    let mut step_cooldown = 0.0f32;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        // ====== INPUT SNAPSHOT (antes de dibujar) ======
        let mouse_pos = rl.get_mouse_position();
        let click_left = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let r_pressed = rl.is_key_pressed(KeyboardKey::KEY_R);
        let m_pressed = rl.is_key_pressed(KeyboardKey::KEY_M); // <- para volver al menú
        let back_pressed = rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE);
        // ====== UPDATE ======
        match state {
            AppState::Menu => {
                sfx.set_music_volume(0.0);
            }
            AppState::Playing => {
                if !won && !dead {
                    {
                        let maze = levels.active_mut();
                        let door_toggled = process_input(&rl, &mut player, maze, dt);
                        maze.update_doors(dt);
                        if door_toggled {
                            sfx.sfx_door(0.8);
                        }
                    }

                    if let Some(e) = enemy.as_mut() {
                        if e.update(levels.active(), &player, dt) {
                            dead = true;
                            sfx.set_music_volume(0.0);
                        }
                    }

                    match levels.check_transition(&player) {
                        Transition::None => {}
                        Transition::NextLevel => {
                            levels.advance_to_next(&mut player);
                            sfx.sfx_door(0.8);
                            unlocked = unlocked.max(levels.current + 1);
                            menu.set_unlocked(unlocked);

                            if levels.current >= 1 {
                                enemy =
                                    Some(Enemy::spawn_from_map_or_far(levels.active(), &player));
                                sfx.on_enemy_spawned(1.2);
                            } else {
                                enemy = None;
                            }
                        }
                        Transition::Won => {
                            won = true;
                            enemy = None;
                            unlocked = total_levels;
                            menu.set_unlocked(unlocked);
                            sfx.set_music_volume(0.10);
                        }
                    }
                }

                // Mantén audio stream
                let enemy_pos = if !won && !dead {
                    enemy.as_ref().map(|e| e.pos)
                } else {
                    None
                };
                let threat_enabled = !won && !dead && enemy.is_some();
                sfx.update(
                    dt,
                    player.pos,
                    enemy_pos,
                    levels.active(),
                    levels.current,
                    threat_enabled,
                );

                // Pasos
                step_cooldown = (step_cooldown - dt).max(0.0);
                let moved = last_player_pos.distance_to(player.pos);
                last_player_pos = player.pos;
                step_accum += moved;
                let speed = if dt > 0.0 { moved / dt } else { 0.0 };
                let step_stride = if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                    22.0
                } else {
                    28.0
                };

                let maze_now = levels.active();
                let bs = maze_now.block_size as f32;
                let ci = (player.pos.x / bs) as i32;
                let cj = (player.pos.y / bs) as i32;
                let on_walkable = match maze_now.tile_at(ci, cj) {
                    '#' | 'A' | 'B' => false,
                    _ => true,
                };

                if speed > 10.0 && on_walkable && step_accum >= step_stride && step_cooldown <= 0.0
                {
                    let vol = (0.35 + 0.65 * (speed / 220.0)).clamp(0.35, 0.9);
                    sfx.sfx_step(vol);
                    step_accum = 0.0;
                    step_cooldown = 0.06;
                }
            }
        }

        // ====== RENDER A FRAMEBUFFER (antes de begin_drawing) ======
        let mut need_scene = matches!(state, AppState::Playing);
        if need_scene {
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
            tex = rl
                .load_texture_from_image(&thread, &framebuffer.color_buffer)
                .unwrap();
        }

        // ====== DRAW ======
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        match state {
            AppState::Menu => {
                match menu.draw_and_pick(mouse_pos, click_left, back_pressed, &mut d) {
                    MenuOutcome::None => {}
                    MenuOutcome::StartLevel(idx) => {
                        levels.set_current(idx, &mut player);
                        unlocked = unlocked.max(idx + 1);
                        menu.set_unlocked(unlocked);
                        enemy = if idx >= 1 {
                            Some(Enemy::spawn_from_map_or_far(levels.active(), &player))
                        } else {
                            None
                        };
                        won = false;
                        dead = false;
                        sfx.on_enemy_spawned(1.0);
                        state = AppState::Playing;
                        want_enter_play = true; // capturar/ocultar cursor tras cerrar el draw
                    }
                }
            }

            AppState::Playing => {
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
                    draw_centered_text(&mut d, "M para volver al menú", 280, 20, Color::RAYWHITE);
                    if m_pressed {
                        sfx.set_music_volume(0.0);
                        menu.goto_main();
                        state = AppState::Menu;
                        want_back_to_menu = true; // soltar/mostrar cursor tras cerrar el draw
                    }
                } else if dead {
                    // === Fondo Game Over (CONTAIN: muestra la imagen completa) ===
                    if let Some(tex) = &game_over_tex {
                        let sw = d.get_screen_width() as f32;
                        let sh = d.get_screen_height() as f32;
                        let iw = tex.width() as f32;
                        let ih = tex.height() as f32;

                        // Contain: sin recortar la imagen
                        let scale = (sw / iw).min(sh / ih);
                        let w = iw * scale;
                        let h = ih * scale;
                        let dst = Rectangle {
                            x: (sw - w) / 2.0,
                            y: (sh - h) / 2.0,
                            width: w,
                            height: h,
                        };
                        let src = Rectangle {
                            x: 0.0,
                            y: 0.0,
                            width: iw,
                            height: ih,
                        };

                        // Fondo oscuro y luego la imagen centrada
                        d.draw_rectangle(
                            0,
                            0,
                            d.get_screen_width(),
                            d.get_screen_height(),
                            Color::BLACK,
                        );
                        d.draw_texture_pro(tex, src, dst, Vector2::zero(), 0.0, Color::WHITE);

                        // franja suave abajo para que se lea el texto
                        d.draw_rectangle(
                            0,
                            (sh - 72.0) as i32,
                            sw as i32,
                            72,
                            Color::new(0, 0, 0, 140),
                        );
                    } else {
                        d.draw_rectangle(
                            0,
                            0,
                            d.get_screen_width(),
                            d.get_screen_height(),
                            Color::BLACK,
                        );
                        draw_centered_text(&mut d, "GAME OVER", 220, 48, Color::MAROON);
                    }

                    // Instrucciones encima de la imagen (calcula Y antes de pasar &mut d)
                    let y = d.get_screen_height() - 44;
                    draw_centered_text(&mut d, "R: reintentar  |  M: menú", y, 24, Color::RAYWHITE);

                    // Input
                    if r_pressed {
                        levels.set_current(levels.current, &mut player);
                        enemy = if levels.current >= 1 {
                            Some(Enemy::spawn_from_map_or_far(levels.active(), &player))
                        } else {
                            None
                        };
                        sfx.on_enemy_spawned(0.8);
                        sfx.set_music_volume(0.0);
                        dead = false;
                    } else if m_pressed {
                        sfx.set_music_volume(0.0);
                        menu.goto_main();
                        state = AppState::Menu;
                        want_back_to_menu = true;
                    }
                }
            }
        }

        // Termina el frame antes de tocar cursor
        drop(d);

        // Aplica cambios de cursor y resetea flags
        if want_enter_play {
            rl.disable_cursor(); // captura/oculta cursor (mouse-look)
            want_enter_play = false;
        }
        if want_back_to_menu {
            rl.enable_cursor(); // muestra cursor en menú
            want_back_to_menu = false;
        }
    }
}
