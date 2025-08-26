use crate::{maze::Maze, player::Player};

pub enum Transition {
    None,
    NextLevel,
    Won,
}

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

    pub fn total_levels(&self) -> usize {
        self.maps.len()
    }

    /// Cambia al índice indicado y coloca al jugador en el spawn de ese nivel.
    pub fn set_current(&mut self, idx: usize, player: &mut Player) {
        assert!(idx < self.maps.len());
        self.current = idx;
        place_player_at_spawn(player, self.active_mut());
    }

    pub fn check_transition(&self, player: &Player) -> Transition {
        let bs = self.active().block_size as f32;
        let i = (player.pos.x / bs) as isize;
        let j = (player.pos.y / bs) as isize;
        let tile = self.active().cell(i, j);

        if tile == 'F' {
            return Transition::Won;
        }
        if tile == 'E' {
            if self.current + 1 >= self.maps.len() {
                Transition::Won
            } else {
                Transition::NextLevel
            }
        } else {
            Transition::None
        }
    }

    pub fn advance_to_next(&mut self, player: &mut Player) {
        let next = self.current + 1;
        assert!(next < self.maps.len());
        self.current = next;
        place_player_at_spawn(player, self.active_mut());

        // empujón para no re-disparar trigger
        let bump = 6.0;
        player.pos.x += player.a.cos() * bump;
        player.pos.y += player.a.sin() * bump;
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
    let bs = maze.block_size as f32;
    player.pos.x = (pi as f32 + 0.5) * bs;
    player.pos.y = (pj as f32 + 0.5) * bs;
}
