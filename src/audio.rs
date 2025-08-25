use crate::maze::Maze;
use raylib::core::audio::Sound as RlSound; // alias cómodo
use raylib::core::audio::{Music, RaylibAudio};
use raylib::prelude::*;

/// Música del monstruo (stream)
const MUSIC_MONSTER_PATH: &str = "sounds/enemy.mp3";

/// SFX (usa WAV u OGG; evita MP3 para Sound)
const SFX_STEP_PATH: &str = "sounds/steps.wav";
const SFX_DOOR_PATH: &str = "sounds/door.wav";

/// Parámetros para mapear distancia -> volumen (capa MONSTER).
#[derive(Clone, Copy)]
pub struct DistanceVolume {
    pub near_px: f32,        // <= aquí ya es “cerca” (máximo)
    pub far_px: f32,         // >= aquí ya es “lejos” (mínimo)
    pub min_vol: f32,        // volumen mínimo (lejos)
    pub max_vol: f32,        // volumen máximo (cerca)
    pub attack: f32,         // suavizado al subir
    pub release: f32,        // suavizado al bajar
    pub occlusion_mult: f32, // multiplicador si NO hay LOS (muro/puerta)
}

impl Default for DistanceVolume {
    fn default() -> Self {
        Self {
            near_px: 3.0 * 48.0, // ~3 celdas
            far_px: 9.0 * 48.0,  // ~9 celdas
            min_vol: 0.00,
            max_vol: 0.95,
            attack: 6.0,
            release: 2.5,
            occlusion_mult: 0.22,
        }
    }
}

/// Conjunto de assets y lógica de audio (SFX en memoria + música MONSTER en stream).
pub struct AudioAssets<'a> {
    // --- SFX (memoria) ---
    step: RlSound<'a>,
    door: RlSound<'a>,

    // --- Música del monstruo (tensión) ---
    music: Music<'a>,
    current_music_vol: f32,
    target_music_vol: f32,
    dist_cfg: DistanceVolume,

    /// Silencio tras spawn de enemigo (segundos).
    silence_timer: f32,
}

impl<'a> AudioAssets<'a> {
    /// Carga SFX (WAV/OGG) y la música de tensión como stream.
    pub fn new(aud: &'a RaylibAudio) -> Result<Self, String> {
        // SFX
        let step = aud.new_sound(SFX_STEP_PATH).map_err(|e| e.to_string())?;
        let door = aud.new_sound(SFX_DOOR_PATH).map_err(|e| e.to_string())?;

        // sanity checks: si frame_count() == 0, la decodificación falló
        if step.frame_count() == 0 {
            return Err(format!(
                "'{}' cargó con 0 frames (usa WAV/OGG y verifica la ruta)",
                SFX_STEP_PATH
            ));
        }
        if door.frame_count() == 0 {
            return Err(format!(
                "'{}' cargó con 0 frames (usa WAV/OGG y verifica la ruta)",
                SFX_DOOR_PATH
            ));
        }

        // MONSTER (stream)
        let music = aud.new_music(MUSIC_MONSTER_PATH).map_err(|e| e.to_string())?;
        music.set_volume(0.0);
        music.play_stream();

        Ok(Self {
            step,
            door,
            music,
            current_music_vol: 0.0,
            target_music_vol: 0.0,
            dist_cfg: DistanceVolume::default(),
            silence_timer: 0.0,
        })
    }

    /// Llama esto cuando el enemigo aparece/despierta.
    pub fn on_enemy_spawned(&mut self, grace_secs: f32) {
        self.silence_timer = self.silence_timer.max(grace_secs.max(0.0));
    }

    /// Permite tunear la curva de distancia para la capa MONSTER.
    pub fn set_distance_volume(&mut self, cfg: DistanceVolume) {
        self.dist_cfg = cfg;
    }

    /// Actualiza la capa MONSTER (stream). Llamar cada frame.
    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vector2,
        enemy_pos: Option<Vector2>,
        maze: &Maze,
        level: usize,
        threat_enabled: bool,
    ) {
        // Mantener stream vivo
        self.music.update_stream();

        // ==== MONSTER (tensión) ====
        let mut vol = 0.0;

        if level >= 1 && threat_enabled {
            if let Some(e_pos) = enemy_pos {
                if self.silence_timer > 0.0 {
                    self.silence_timer = (self.silence_timer - dt).max(0.0);
                } else {
                    vol = distance_to_volume(player_pos.distance_to(e_pos), self.dist_cfg);
                    if !has_los_audio(maze, player_pos, e_pos) {
                        vol *= self.dist_cfg.occlusion_mult;
                    }
                }
            }
        }

        self.target_music_vol = vol;
        self.smooth_monster(dt);
    }

    fn smooth_monster(&mut self, dt: f32) {
        let rate = if self.target_music_vol > self.current_music_vol {
            self.dist_cfg.attack
        } else {
            self.dist_cfg.release
        };
        let k = 1.0 - (-dt * rate).exp();
        self.current_music_vol += (self.target_music_vol - self.current_music_vol) * k;
        self.current_music_vol = self.current_music_vol.clamp(0.0, 1.0);
        self.music.set_volume(self.current_music_vol);
    }

    // ===================== Control global =====================

    pub fn pause_music(&self)  { self.music.pause_stream(); }
    pub fn resume_music(&self) { self.music.resume_stream(); }
    pub fn stop_music(&self)   { self.music.stop_stream(); }

    /// Compat: controla SOLO la capa MONSTER (como en la versión anterior).
    pub fn set_music_volume(&mut self, v: f32) {
        self.current_music_vol = v.clamp(0.0, 1.0);
        self.target_music_vol = self.current_music_vol;
        self.music.set_volume(self.current_music_vol);
    }

    // ===================== Disparadores SFX =====================

    /// Paso: usa el sample base (sin alias) y lo re-dispara limpiamente.
    pub fn sfx_step(&self, volume: f32) {
        if !self.step.is_sound_valid() || self.step.frame_count() == 0 {
            eprintln!("[audio] steps WAV no válido o vacío: {}", SFX_STEP_PATH);
            return;
        }
        self.step.set_volume(volume.clamp(0.0, 1.0));
        if self.step.is_playing() { self.step.stop(); }
        self.step.play();
    }

    /// Puerta: igual que arriba.
    pub fn sfx_door(&self, volume: f32) {
        if !self.door.is_sound_valid() || self.door.frame_count() == 0 {
            eprintln!("[audio] door WAV no válido o vacío: {}", SFX_DOOR_PATH);
            return;
        }
        self.door.set_volume(volume.clamp(0.0, 1.0));
        if self.door.is_playing() { self.door.stop(); }
        self.door.play();
    }
}

/// Mapea distancia (px) a volumen usando smoothstep (0..1).
fn distance_to_volume(d_px: f32, cfg: DistanceVolume) -> f32 {
    let d = d_px.clamp(0.0, cfg.far_px);
    let t = if d <= cfg.near_px {
        1.0
    } else {
        let r = (d - cfg.near_px) / (cfg.far_px - cfg.near_px + 1e-6);
        (1.0 - r).clamp(0.0, 1.0)
    };
    let smooth = t * t * (3.0 - 2.0 * t);
    cfg.min_vol + (cfg.max_vol - cfg.min_vol) * smooth
}

/// Línea de vista para audio: si hay celda bloqueante en el trazo → NO LOS.
fn has_los_audio(maze: &Maze, from: Vector2, to: Vector2) -> bool {
    let bs = maze.block_size as f32;
    let total = from.distance_to(to);
    if total < 1.0 { return true; }
    let step = (bs * 0.25).max(2.0); // denso → más preciso
    let dir = (to - from) / total;
    let mut d = 0.0f32;

    while d <= total {
        let p = from + dir * d;
        let ci = (p.x / bs) as isize;
        let cj = (p.y / bs) as isize;
        if maze.is_blocking_at(ci, cj) {
            return false;
        }
        d += step;
    }
    true
}
