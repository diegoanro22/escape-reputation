use raylib::core::audio::{Music, RaylibAudio, Sound};
use raylib::prelude::*;

/// Rutas por defecto (cámbialas a tu estructura de assets)
const MUSIC_PATH: &str = "sounds/taylor.mp3";
// const SFX_STEP_PATH: &str = "assets/sfx/step.wav";
// const SFX_HIT_PATH: &str = "assets/sfx/hit.wav";
// const SFX_PICKUP_PATH: &str = "assets/sfx/pickup.wav";
// const SFX_DOOR_PATH: &str = "assets/sfx/door.wav";

/// Parámetros para mapear distancia -> volumen.
#[derive(Clone, Copy)]
pub struct DistanceVolume {
    /// Distancia (px) a partir de la cual ya suena "fuerte".
    pub near_px: f32,
    /// Distancia (px) a partir de la cual el volumen llega al mínimo.
    pub far_px: f32,
    /// Volumen mínimo cuando está muy lejos.
    pub min_vol: f32,
    /// Volumen máximo cuando está muy cerca.
    pub max_vol: f32,
    /// Rápidez de subida del volumen (attack), en 1/seg.
    pub attack: f32,
    /// Rápidez de bajada del volumen (release), en 1/seg.
    pub release: f32,
}

impl Default for DistanceVolume {
    fn default() -> Self {
        Self {
            // Con tus mapas (block_size=48), 10 celdas ≈ 480 px.
            // Puedes empezar a "asustar" ~6 celdas y apagar a ~18 celdas.
            near_px: 6.0 * 48.0,
            far_px: 18.0 * 48.0,
            min_vol: 0.10,
            max_vol: 0.95,
            attack: 6.0,  // sube rápido si el enemigo se acerca
            release: 2.5, // baja más despacio si se aleja
        }
    }
}

/// Conjunto de assets y lógica de audio.
pub struct AudioAssets<'a> {
    // --- SFX ---
    // step: Sound<'a>,
    // hit: Sound<'a>,
    // pickup: Sound<'a>,
    // door: Sound<'a>,

    // --- Música principal (streaming) ---
    music: Music<'a>,
    /// Volumen actual aplicado a la música.
    current_music_vol: f32,
    /// Target al que queremos llegar según la distancia.
    target_music_vol: f32,
    /// Parámetros de mapeo distancia->volumen.
    dist_cfg: DistanceVolume,
}

impl<'a> AudioAssets<'a> {
    /// Carga audio. Llama una sola vez después de crear `RaylibAudio`.
    pub fn new(aud: &'a RaylibAudio) -> Result<Self, String> {
        // SFX
        // let step = aud.new_sound(SFX_STEP_PATH).map_err(|e| e.to_string())?;
        // let hit = aud.new_sound(SFX_HIT_PATH).map_err(|e| e.to_string())?;
        // let pickup = aud.new_sound(SFX_PICKUP_PATH).map_err(|e| e.to_string())?;
        // let door = aud.new_sound(SFX_DOOR_PATH).map_err(|e| e.to_string())?;

        // Música (streaming)
        let music = aud.new_music(MUSIC_PATH).map_err(|e| e.to_string())?;
        music.set_volume(0.0); // arrancamos silencioso (sube con la distancia)
        music.play_stream(); // reproducir ya; haremos update cada frame

        Ok(Self {
            // step,
            // hit,
            // pickup,
            // door,
            music,
            current_music_vol: 0.0,
            target_music_vol: 0.0,
            dist_cfg: DistanceVolume::default(),
        })
    }

    /// Cambia parámetros de distancia->volumen si quieres tunear en runtime.
    pub fn set_distance_volume(&mut self, cfg: DistanceVolume) {
        self.dist_cfg = cfg;
    }

    /// Llamar **cada frame**.
    ///
    /// - `dt`: deltaTime del frame.
    /// - `player_pos`: posición del jugador (px).
    /// - `enemy_pos`: `Some(pos)` si hay enemigo; `None` si no.
    pub fn update(&mut self, dt: f32, player_pos: Vector2, enemy_pos: Option<Vector2>) {
        // Mantener buffers del stream
        self.music.update_stream();

        // 1) calcular target por distancia
        self.target_music_vol = if let Some(e_pos) = enemy_pos {
            let d = player_pos.distance_to(e_pos);
            distance_to_volume(d, self.dist_cfg)
        } else {
            // sin enemigo presente → volumen base bajo
            self.dist_cfg.min_vol
        };

        // 2) suavizar hacia el target (attack/release)
        let rate = if self.target_music_vol > self.current_music_vol {
            self.dist_cfg.attack
        } else {
            self.dist_cfg.release
        };
        // aproximación exponencial suave: v += (target-v) * (1 - e^(-dt*rate))
        let k = 1.0 - (-dt * rate).exp();
        self.current_music_vol += (self.target_music_vol - self.current_music_vol) * k;
        self.current_music_vol = self.current_music_vol.clamp(0.0, 1.0);

        self.music.set_volume(self.current_music_vol);
    }

    // ===================== Disparadores SFX =====================

    // /// Paso del jugador (usa alias para que no se corte si suenan seguidos).
    // pub fn sfx_step(&self, volume: f32) {
    //     if let Ok(alias) = self.step.alias() {
    //         alias.set_volume(volume.clamp(0.0, 1.0));
    //         alias.play();
    //     }
    // }

    // /// Golpe/zarpazo del enemigo.
    // pub fn sfx_hit(&self, volume: f32) {
    //     if let Ok(alias) = self.hit.alias() {
    //         alias.set_volume(volume.clamp(0.0, 1.0));
    //         alias.play();
    //     }
    // }

    // /// Recoger objeto / suministros.
    // pub fn sfx_pickup(&self, volume: f32) {
    //     if let Ok(alias) = self.pickup.alias() {
    //         alias.set_volume(volume.clamp(0.0, 1.0));
    //         alias.play();
    //     }
    // }

    // /// Puerta (abrir/cerrar). Puedes mandar distinto volumen según la distancia.
    // pub fn sfx_door(&self, volume: f32) {
    //     if let Ok(alias) = self.door.alias() {
    //         alias.set_volume(volume.clamp(0.0, 1.0));
    //         alias.play();
    //     }
    // }

    // ===================== Control global =====================

    /// Pausa/Reanuda música (útil en pausa o al ganar/perder).
    pub fn pause_music(&self) {
        self.music.pause_stream();
    }
    pub fn resume_music(&self) {
        self.music.resume_stream();
    }
    pub fn stop_music(&self) {
        self.music.stop_stream();
    }

    /// Por si quieres forzar el volumen de música puntualmente.
    pub fn set_music_volume(&mut self, v: f32) {
        self.current_music_vol = v.clamp(0.0, 1.0);
        self.target_music_vol = self.current_music_vol;
        self.music.set_volume(self.current_music_vol);
    }
}

/// Mapea distancia (px) a volumen [min_vol, max_vol] mediante una curva suave.
fn distance_to_volume(d_px: f32, cfg: DistanceVolume) -> f32 {
    let d = d_px.clamp(0.0, cfg.far_px);
    // 0..1 donde 0 = lejos, 1 = cerca
    let t = if d <= cfg.near_px {
        1.0
    } else {
        let r = (d - cfg.near_px) / (cfg.far_px - cfg.near_px + 1e-6);
        (1.0 - r).clamp(0.0, 1.0)
    };
    // curva smoothstep (más natural que lineal)
    let smooth = t * t * (3.0 - 2.0 * t);
    cfg.min_vol + (cfg.max_vol - cfg.min_vol) * smooth
}
