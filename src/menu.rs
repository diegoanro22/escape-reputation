use crate::draw_utils::draw_centered_text;
use raylib::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Main,
    LevelSelect,
    HowTo,
}

pub enum MenuOutcome {
    None,
    StartLevel(usize),
}

pub struct Menu {
    bg: Option<Texture2D>,
    screen: Screen,
    unlocked: usize,
    total: usize,
}

impl Menu {
    // Cambia la ruta si lo necesitas
    const BG_PATH: &'static str = "assets/menu.png";

    pub fn new(rl: &mut RaylibHandle, th: &RaylibThread, total_levels: usize) -> Self {
        let bg = rl.load_texture(th, Self::BG_PATH).ok(); // si no existe, se dibuja un degradado
        Self {
            bg,
            screen: Screen::Main,
            unlocked: 1,
            total: total_levels.max(1),
        }
    }

    pub fn goto_main(&mut self) {
        self.screen = Screen::Main;
    }
    pub fn set_unlocked(&mut self, count: usize) {
        self.unlocked = count.clamp(1, self.total);
    }

    /// Dibuja y procesa *con los inputs ya calculados*.
    pub fn draw_and_pick(
        &mut self,
        mouse_pos: Vector2,
        click_left: bool,
        back_pressed: bool, // <— Backspace
        d: &mut RaylibDrawHandle,
    ) -> MenuOutcome {
        self.draw_background(d);
        match self.screen {
            Screen::Main => self.draw_main(mouse_pos, click_left, d),
            Screen::LevelSelect => self.draw_level_select(mouse_pos, click_left, back_pressed, d),
            Screen::HowTo => self.draw_howto(back_pressed, d),
        }
    }

    fn draw_background(&self, d: &mut RaylibDrawHandle) {
        if let Some(tex) = &self.bg {
            let dst = Rectangle {
                x: 0.0,
                y: 0.0,
                width: d.get_screen_width() as f32,
                height: d.get_screen_height() as f32,
            };
            let src = Rectangle {
                x: 0.0,
                y: 0.0,
                width: tex.width() as f32,
                height: tex.height() as f32,
            };
            d.draw_texture_pro(tex, src, dst, Vector2::zero(), 0.0, Color::WHITE);
            // oscurecer levemente sin tapar
            d.draw_rectangle(0, 0, d.get_screen_width(), 140, Color::new(0, 0, 0, 40));
        } else {
            d.draw_rectangle_gradient_v(
                0,
                0,
                d.get_screen_width(),
                d.get_screen_height(),
                Color::new(7, 7, 12, 255),
                Color::new(18, 18, 22, 255),
            );
        }
    }

    /// Barra superior: botones “píldora” en fila, transparentes (solo hover).
    fn draw_main(
        &mut self,
        mouse_pos: Vector2,
        click_left: bool,
        d: &mut RaylibDrawHandle,
    ) -> MenuOutcome {
        // Layout
        let y = 18.0;
        let mut x = 18.0;
        let fs = 26; // font size
        let pad_x = 22.0;
        let pad_y = 10.0;
        let gap = 12.0;

        // Empezar
        let (clicked, w) = pill_button(d, mouse_pos, click_left, x, y, "Empezar", fs, pad_x, pad_y);
        if clicked {
            let level = self.unlocked.saturating_sub(1);
            return MenuOutcome::StartLevel(level);
        }
        x += w + gap;

        // Seleccionar nivel
        let (clicked, w) = pill_button(
            d,
            mouse_pos,
            click_left,
            x,
            y,
            "Seleccionar nivel",
            fs,
            pad_x,
            pad_y,
        );
        if clicked {
            self.screen = Screen::LevelSelect;
        }
        x += w + gap;

        // Cómo jugar
        let (clicked, _w) = pill_button(
            d,
            mouse_pos,
            click_left,
            x,
            y,
            "Cómo jugar",
            fs,
            pad_x,
            pad_y,
        );
        if clicked {
            self.screen = Screen::HowTo;
        }

        MenuOutcome::None
    }

    fn draw_level_select(
        &mut self,
        mouse_pos: Vector2,
        click_left: bool,
        back_pressed: bool,
        d: &mut RaylibDrawHandle,
    ) -> MenuOutcome {
        draw_centered_text(d, "Seleccionar nivel", 64, 30, Color::RAYWHITE);

        let cols = 5usize;
        let bw = 110.0;
        let bh = 64.0;
        let gap = 16.0;
        let total_w = cols as f32 * bw + (cols as f32 - 1.0) * gap;
        let start_x = (d.get_screen_width() as f32 - total_w) / 2.0;
        let start_y = 120.0;

        for i in 0..self.total {
            let row = (i / cols) as i32;
            let col = (i % cols) as i32;
            let x = start_x + col as f32 * (bw + gap);
            let y = start_y + row as f32 * (bh + gap);
            let r = rect(x, y, bw, bh);
            let label = format!("Nivel {}", i);

            if i < self.unlocked {
                if button_box(d, mouse_pos, click_left, r, &label) {
                    return MenuOutcome::StartLevel(i);
                }
            } else {
                draw_button_box(d, r, point_in_rect(mouse_pos, r));
                let tw = d.measure_text("BLOQUEADO", 16);
                d.draw_text(
                    "BLOQUEADO",
                    (x + (bw - tw as f32) / 2.0) as i32,
                    (y + 12.0) as i32,
                    16,
                    Color::LIGHTGRAY,
                );
                let tl = d.measure_text(&label, 16);
                d.draw_text(
                    &label,
                    (x + (bw - tl as f32) / 2.0) as i32,
                    (y + 36.0) as i32,
                    16,
                    Color::GRAY,
                );
            }
        }

        d.draw_text(
            "BACKSPACE: regresar",
            20,
            d.get_screen_height() - 28,
            18,
            Color::LIGHTGRAY,
        );
        if back_pressed {
            self.screen = Screen::Main;
        }
        MenuOutcome::None
    }

    fn draw_howto(&mut self, back_pressed: bool, d: &mut RaylibDrawHandle) -> MenuOutcome {
        draw_centered_text(d, "Cómo jugar", 64, 30, Color::RAYWHITE);
        let lines = [
            "WASD / Flechas: mover",
            "Mouse: girar cámara",
            "Shift o LT/RT: correr",
            "E o Botón A: abrir/cerrar puertas",
            "Sube de nivel por 'E'; en el último busca 'F' para escapar.",
            "Si te atrapan: pulsa R para reintentar.",
            "M: volver al menú en juego",
        ];
        let mut y = 120;
        for l in lines {
            draw_centered_text(d, l, y, 22, Color::RAYWHITE);
            y += 30;
        }
        d.draw_text(
            "BACKSPACE: regresar",
            20,
            d.get_screen_height() - 28,
            18,
            Color::LIGHTGRAY,
        );
        if back_pressed {
            self.screen = Screen::Main;
        }
        MenuOutcome::None
    }
}

/* ---------- helpers UI ---------- */

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rectangle {
    Rectangle {
        x,
        y,
        width: w,
        height: h,
    }
}

/// Botón “píldora” (transparente por defecto, con fondo suave al hover).
/// Devuelve (clicked, width_usada)
fn pill_button(
    d: &mut RaylibDrawHandle,
    mouse_pos: Vector2,
    click_left: bool,
    x: f32,
    y: f32,
    label: &str,
    font_size: i32,
    pad_x: f32,
    pad_y: f32,
) -> (bool, f32) {
    let tw = d.measure_text(label, font_size) as f32;
    let w = tw + pad_x * 2.0;
    let h = font_size as f32 + pad_y * 2.0;
    let r = rect(x, y, w, h);

    let hover = point_in_rect(mouse_pos, r);

    if hover {
        // fondo muy sutil para no tapar
        d.draw_rectangle_rounded(r, 0.6, 12, Color::new(0, 0, 0, 110));
        d.draw_rectangle_rounded_lines(r, 0.6, 12, Color::new(200, 200, 200, 130));
    }

    // texto con ligera sombra
    d.draw_text(
        label,
        (x + pad_x + 1.0) as i32,
        (y + pad_y + 1.0) as i32,
        font_size,
        Color::new(0, 0, 0, 120),
    );
    d.draw_text(
        label,
        (x + pad_x) as i32,
        (y + pad_y) as i32,
        font_size,
        Color::RAYWHITE,
    );

    (hover && click_left, w)
}

/// Botón tipo caja para el selector de niveles (mantengo estilo anterior).
fn button_box(
    d: &mut RaylibDrawHandle,
    mouse_pos: Vector2,
    click_left: bool,
    r: Rectangle,
    label: &str,
) -> bool {
    let hover = point_in_rect(mouse_pos, r);
    draw_button_box(d, r, hover);
    let tw = d.measure_text(label, 24);
    d.draw_text(
        label,
        (r.x + (r.width - tw as f32) / 2.0) as i32,
        (r.y + 14.0) as i32,
        24,
        Color::RAYWHITE,
    );
    hover && click_left
}

fn draw_button_box(d: &mut RaylibDrawHandle, r: Rectangle, hover: bool) {
    let base = if hover {
        Color::new(60, 60, 70, 180)
    } else {
        Color::new(40, 40, 50, 150)
    };
    d.draw_rectangle_rounded(r, 0.20, 8, base);
    d.draw_rectangle_rounded_lines(r, 0.20, 8, Color::GRAY);
}

fn point_in_rect(p: Vector2, r: Rectangle) -> bool {
    p.x >= r.x && p.x <= r.x + r.width && p.y >= r.y && p.y <= r.y + r.height
}
