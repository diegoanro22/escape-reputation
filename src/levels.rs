use crate::{maze::Maze, player::Player};

pub struct Levels {
    pub maps: Vec<Maze>,
    pub current: usize,
}

impl Levels {
    pub fn new(maps: Vec<Maze>) -> Self {
        assert!(!maps.is_empty(), "necesitas al menos 1 nivel");
        Self { maps, current: 0 }
    }

    #[inline]
    pub fn active(&self) -> &Maze {
        &self.maps[self.current]
    }

    #[inline]
    pub fn active_mut(&mut self) -> &mut Maze {
        &mut self.maps[self.current]
    }

    /// Si el jugador pisa 'E' (salida), avanza al siguiente nivel y lo coloca en el 'P' del nuevo.
    pub fn try_advance_on_exit(&mut self, player: &mut Player) -> bool {
        let bs = self.active().block_size as f32;
        let i = (player.pos.x / bs) as isize;
        let j = (player.pos.y / bs) as isize;

        let tile = self.active().cell(i, j);
        if tile != 'E' {
            return false;
        }

        let next = self.current + 1;
        if next >= self.maps.len() {
            return false;
        }

        self.current = next;
        place_player_at_spawn(player, self.active_mut());

        // empujón para no re-disparar el trigger
        let bump = 6.0;
        player.pos.x += player.a.cos() * bump;
        player.pos.y += player.a.sin() * bump;

        true
    }
}

fn place_player_at_spawn(player: &mut Player, maze: &mut Maze) {
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
    let (pi, pj) = spawn.expect("El nivel no tiene 'P' (spawn)");
    maze.grid[pj][pi] = '.'; // limpia a piso

    let bs = maze.block_size as f32;
    player.pos.x = (pi as f32 + 0.5) * bs;
    player.pos.y = (pj as f32 + 0.5) * bs;
}
