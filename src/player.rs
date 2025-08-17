// src/player.rs
use crate::maze::Maze;
use raylib::prelude::*;

pub struct Player {
    pub pos: Vector2, // en píxeles
    pub a: f32,       // ángulo (radianes)
    pub fov: f32,     // field of view
}

impl Player {
    pub fn from_maze(maze: &mut Maze, a: f32, fov: f32) -> Self {
        // busca 'P'
        let mut spawn = None;
        for (j, row) in maze.grid.iter().enumerate() {
            for (i, &c) in row.iter().enumerate() {
                if c == 'P' {
                    spawn = Some((i, j));
                    break;
                }
            }
            if spawn.is_some() {
                break;
            }
        }
        let (pi, pj) = spawn.expect("El maze debe tener un 'P'");

        // limpia la celda para que sea piso
        maze.grid[pj][pi] = '.';

        // posición en píxeles (centro de la celda)
        let x = (pi as f32 + 0.5) * maze.block_size as f32;
        let y = (pj as f32 + 0.5) * maze.block_size as f32;

        Self {
            pos: Vector2::new(x, y),
            a,
            fov,
        }
    }
}
