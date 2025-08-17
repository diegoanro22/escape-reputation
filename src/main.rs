mod framebuffer;
mod maze;

use framebuffer::FrameBuffer;
use maze::{Maze, render_maze_2d};
use raylib::prelude::*;

fn main() {
    let width = 800;
    let height = 600;

    let mut framebuffer = FrameBuffer::new(width, height, Color::BLACK);

    // Carga el maze y elige un tamaño de celda que quepa en pantalla
    let maze = Maze::load_from_file("maze.txt", 48).expect("no pude cargar maze.txt");

    // Render top-down
    render_maze_2d(&mut framebuffer, &maze);

    // Exporta para revisar visualmente
    framebuffer.render_to_file("maze_debug.png").unwrap();
    println!("Listo: maze_debug.png");
}
