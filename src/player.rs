use crate::maze::Maze;
use raylib::prelude::*;

pub struct Player {
    pub pos: Vector2,    // píxeles
    pub a: f32,          // ángulo (radianes)
    pub fov: f32,        // field of view
    pub move_speed: f32, // px/seg
    pub rot_speed: f32,  // rad/seg
}

impl Player {
    pub fn from_maze(maze: &mut Maze, a: f32, fov: f32) -> Self {
        // encuentra 'P'
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

        // limpia 'P' para que sea piso
        maze.grid[pj][pi] = '.';

        // centro de la celda en píxeles
        let x = (pi as f32 + 0.5) * maze.block_size as f32;
        let y = (pj as f32 + 0.5) * maze.block_size as f32;

        Self {
            pos: Vector2::new(x, y),
            a,
            fov,
            move_speed: 120.0, 
            rot_speed: 2.5,   
        }
    }
}
