mod caster;
mod draw_utils;
mod framebuffer;
mod maze;
mod player;
mod render2d;

use caster::cast_ray_topdown;
use draw_utils::draw_disc;
use framebuffer::FrameBuffer;
use maze::Maze;
use player::Player;
use render2d::render_maze_2d;

use raylib::prelude::*;

fn main() {
    let mut framebuffer = FrameBuffer::new(800, 600, Color::BLACK);

    let mut mz = Maze::load_from_file("maze.txt", 48).expect("maze.txt no cargó");

    let p = Player::from_maze(
        &mut mz,
        std::f32::consts::FRAC_PI_3,
        std::f32::consts::FRAC_PI_3,
    );

    render_maze_2d(&mut framebuffer, &mz);
    draw_disc(
        &mut framebuffer,
        p.pos.x as i32,
        p.pos.y as i32,
        6,
        Color::GOLD,
    );

    let _hit = cast_ray_topdown(&mut framebuffer, &mz, &p, p.a, true);

    framebuffer.render_to_file("maze_with_ray.png").unwrap();
    println!("Listo: maze_with_ray.png");
}
