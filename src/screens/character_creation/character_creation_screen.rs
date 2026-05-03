use crate::core::app_state::{AppState, CharacterData, Gender};
use crate::graphics::ui_renderer::batch::{Color, DrawBatch, Rect};
use crate::ui::panel::Panel;

const STEP_TITLES: [&str; 10] = [
    "ПОЛ",
    "РОСТ",
    "ЦВЕТ КОЖИ",
    "ЛИЦО",
    "ПРИЧЁСКА",
    "ЦВЕТ ВОЛОС",
    "УНИВЕРСИТЕТ",
    "СПЕЦИАЛЬНОСТЬ",
    "РЕГИОН",
    "ГОТОВО",
];

const SKIN_COLORS: [[f32; 3]; 5] = [
    [0.96, 0.85, 0.73],
    [0.87, 0.70, 0.55],
    [0.73, 0.53, 0.37],
    [0.57, 0.37, 0.24],
    [0.42, 0.26, 0.16],
];

const HAIR_COLORS: [[f32; 3]; 5] = [
    [0.12, 0.08, 0.05],
    [0.30, 0.20, 0.10],
    [0.55, 0.35, 0.15],
    [0.85, 0.70, 0.40],
    [0.20, 0.12, 0.06],
];

const UNIVERSITIES: [&str; 4] = [
    "МГУ", "МГТУ", "СПбГУ", "МИФИ",
];

const SPECIALTIES: [&str; 4] = [
    "Инженерия", "Механика", "Экономика", "Право",
];

const CAPITAL_TABLE: [[f64; 4]; 4] = [
    [40000.0, 55000.0, 70000.0, 90000.0],
    [35000.0, 50000.0, 65000.0, 80000.0],
    [45000.0, 60000.0, 80000.0, 100000.0],
    [30000.0, 45000.0, 60000.0, 75000.0],
];

const REGIONS: [&str; 4] = [
    "Центр", "Промзона", "Спальный", "Пригород",
];

const REGION_POS: [[f64; 3]; 4] = [
    [0.0, 0.0, 0.0],
    [500.0, 0.0, 0.0],
    [-300.0, 200.0, 0.0],
    [-800.0, -100.0, 0.0],
];

const FS_TITLE: f32 = 24.0;
const FS_BTN: f32 = 18.0;
const FS_LABEL: f32 = 15.0;
const FS_HINT: f32 = 13.0;

pub struct CharacterCreationScreen {
    current_step: u8,
    total_steps: u8,
    data: CharacterData,
    hovered: HoverTarget,
    dragging: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum HoverTarget {
    None, Back, Next, Option(usize),
}

impl CharacterCreationScreen {
    pub fn new() -> Self {
        Self {
            current_step: 1,
            total_steps: 10,
            data: CharacterData::default(),
            hovered: HoverTarget::None,
            dragging: false,
        }
    }

    pub fn current_step(&self) -> u8 { self.current_step }

    fn panel(&self, sw: f32, sh: f32) -> Rect {
        let w = 700.0;
        let h = 420.0;
        Rect { x: (sw - w) / 2.0, y: (sh - h) / 2.0 - 10.0, w, h }
    }

    fn preview_rect(&self, sw: f32, sh: f32) -> Rect {
        let p = self.panel(sw, sh);
        Rect { x: p.x + 10.0, y: p.y + 55.0, w: 170.0, h: p.h - 75.0 }
    }

    fn options_area(&self, sw: f32, sh: f32) -> Rect {
        let p = self.panel(sw, sh);
        let pv = self.preview_rect(sw, sh);
        Rect { x: pv.x + pv.w + 15.0, y: pv.y, w: p.w - pv.w - 25.0, h: pv.h }
    }

    fn btn_back(&self, sw: f32, sh: f32) -> Rect {
        let p = self.panel(sw, sh);
        Rect { x: p.x + 15.0, y: p.y + p.h + 12.0, w: 140.0, h: 42.0 }
    }

    fn btn_next(&self, sw: f32, sh: f32) -> Rect {
        let p = self.panel(sw, sh);
        Rect { x: p.x + p.w - 155.0, y: p.y + p.h + 12.0, w: 140.0, h: 42.0 }
    }

    fn selected_idx(&self) -> usize {
        let s = (self.current_step - 1) as usize;
        match s {
            0 => if self.data.gender == Gender::Male { 0 } else { 1 },
            2 => self.data.skin_color as usize,
            5 => HAIR_COLORS.iter().position(|c| *c == self.data.hair_color).unwrap_or(0),
            6 => UNIVERSITIES.iter().position(|u| u == &self.data.university_id).unwrap_or(0),
            7 => SPECIALTIES.iter().position(|s| s == &self.data.specialty).unwrap_or(0),
            8 => REGIONS.iter().position(|r| r == &self.data.start_region).unwrap_or(0),
            _ => 0,
        }
    }

    fn update_capital(&mut self) {
        let ui = UNIVERSITIES.iter().position(|u| u == &self.data.university_id).unwrap_or(0);
        let si = SPECIALTIES.iter().position(|s| s == &self.data.specialty).unwrap_or(0);
        self.data.start_capital = CAPITAL_TABLE[ui][si];
    }

    fn update_hover(&mut self, mx: f32, my: f32, sw: f32, sh: f32) {
        if self.btn_back(sw, sh).contains(mx, my) { self.hovered = HoverTarget::Back; return; }
        if self.btn_next(sw, sh).contains(mx, my) { self.hovered = HoverTarget::Next; return; }

        let s = (self.current_step - 1) as usize;
        match s {
            0 => {
                let a = self.options_area(sw, sh);
                let bw = a.w * 0.45; let bh = 48.0;
                let gap = a.w - bw * 2.0;
                for i in 0..2 {
                    let r = Rect { x: a.x + i as f32 * (bw + gap), y: a.y + a.h / 2.0 - bh / 2.0, w: bw, h: bh };
                    if r.contains(mx, my) { self.hovered = HoverTarget::Option(i); return; }
                }
            }
            2 | 5 => {
                let a = self.options_area(sw, sh);
                let bw = 64.0; let gap = 12.0;
                let total = 5.0 * bw + 4.0 * gap;
                let sx = a.x + (a.w - total) / 2.0;
                let cy = a.y + a.h / 2.0 - bw / 2.0 - 16.0;
                for i in 0..5 {
                    let r = Rect { x: sx + i as f32 * (bw + gap), y: cy, w: bw, h: bw };
                    if r.contains(mx, my) { self.hovered = HoverTarget::Option(i); return; }
                }
            }
            1 => {
                let a = self.options_area(sw, sh);
                let slider_w = a.w * 0.8;
                let sx = a.x + (a.w - slider_w) / 2.0;
                let sy = a.y + a.h / 2.0;
                if mx >= sx - 10.0 && mx <= sx + slider_w + 10.0 && my >= sy - 20.0 && my <= sy + 20.0 {
                    self.hovered = HoverTarget::Option(0); return;
                }
            }
            6 | 7 | 8 => {
                let a = self.options_area(sw, sh);
                let bw = a.w * 0.45; let bh = 44.0;
                let gap = a.w - bw * 2.0;
                let cols = if s == 8 { 4 } else { 4 };
                let total = cols as f32 * bw + (cols as f32 - 1.0) * if cols > 2 { gap / 3.0 } else { gap };
                let sx = a.x + (a.w - total) / 2.0;
                let cy = a.y + a.h / 2.0 - bh / 2.0;
                let col_gap = if cols > 2 { gap / 3.0 } else { gap };
                for i in 0..cols {
                    let r = Rect { x: sx + i as f32 * (bw + col_gap), y: cy, w: bw, h: bh };
                    if r.contains(mx, my) { self.hovered = HoverTarget::Option(i); return; }
                }
            }
            9 => {
                let a = self.options_area(sw, sh);
                let bw = a.w * 0.22; let bh = 44.0;
                let gap = (a.w - 4.0 * bw) / 3.0;
                let cy = a.y + a.h / 2.0 - bh / 2.0 + 50.0;
                for i in 0..4 {
                    let r = Rect { x: a.x + i as f32 * (bw + gap), y: cy, w: bw, h: bh };
                    if r.contains(mx, my) { self.hovered = HoverTarget::Option(i); return; }
                }
            }
            _ => {}
        }
        self.hovered = HoverTarget::None;
    }

    pub fn update(
        &mut self,
        mouse_x: f32,
        mouse_y: f32,
        mouse_just_pressed: bool,
        mouse_held: bool,
        screen_w: f32,
        screen_h: f32,
    ) -> Option<AppState> {
        self.update_hover(mouse_x, mouse_y, screen_w, screen_h);

        let s = (self.current_step - 1) as usize;
        if s == 1 && (self.dragging || mouse_held) {
            let a = self.options_area(screen_w, screen_h);
            let slider_w = a.w * 0.8;
            let sx = a.x + (a.w - slider_w) / 2.0;
            if mouse_x >= sx - 10.0 && mouse_x <= sx + slider_w + 10.0 {
                let t = ((mouse_x - sx) / slider_w).clamp(0.0, 1.0);
                self.data.height_m = 1.50 + t * 0.50;
                self.dragging = true;
            }
        }
        if !mouse_held { self.dragging = false; }

        if mouse_just_pressed {
            if self.btn_back(screen_w, screen_h).contains(mouse_x, mouse_y) {
                if self.current_step > 1 { self.current_step -= 1; }
                return None;
            }
            if self.btn_next(screen_w, screen_h).contains(mouse_x, mouse_y) {
                if s == 9 {
                    return Some(AppState::Loading { character_data: Box::new(self.data.clone()) });
                }
                if self.current_step < self.total_steps { self.current_step += 1; }
                return None;
            }

            match s {
                0 => {
                    let a = self.options_area(screen_w, screen_h);
                    let bw = a.w * 0.45; let bh = 48.0;
                    let gap = a.w - bw * 2.0;
                    for i in 0..2 {
                        let r = Rect { x: a.x + i as f32 * (bw + gap), y: a.y + a.h / 2.0 - bh / 2.0, w: bw, h: bh };
                        if r.contains(mouse_x, mouse_y) {
                            self.data.gender = if i == 0 { Gender::Male } else { Gender::Female };
                        }
                    }
                }
                2 => {
                    let a = self.options_area(screen_w, screen_h);
                    let bw = 64.0; let gap = 12.0;
                    let total = 5.0 * bw + 4.0 * gap;
                    let sx = a.x + (a.w - total) / 2.0;
                    let cy = a.y + a.h / 2.0 - bw / 2.0 - 16.0;
                    for i in 0..5 {
                        let r = Rect { x: sx + i as f32 * (bw + gap), y: cy, w: bw, h: bw };
                        if r.contains(mouse_x, mouse_y) { self.data.skin_color = i as u8; }
                    }
                }
                5 => {
                    let a = self.options_area(screen_w, screen_h);
                    let bw = 64.0; let gap = 12.0;
                    let total = 5.0 * bw + 4.0 * gap;
                    let sx = a.x + (a.w - total) / 2.0;
                    let cy = a.y + a.h / 2.0 - bw / 2.0 - 16.0;
                    for i in 0..5 {
                        let r = Rect { x: sx + i as f32 * (bw + gap), y: cy, w: bw, h: bw };
                        if r.contains(mouse_x, mouse_y) { self.data.hair_color = HAIR_COLORS[i]; }
                    }
                }
                6 => {
                    let a = self.options_area(screen_w, screen_h);
                    let bw = a.w * 0.45; let bh = 44.0;
                    let gap = a.w - bw * 2.0;
                    let col_gap = gap / 3.0;
                    for i in 0..4 {
                        let r = Rect { x: a.x + i as f32 * (bw + col_gap), y: a.y + a.h / 2.0 - bh / 2.0, w: bw, h: bh };
                        if r.contains(mouse_x, mouse_y) {
                            self.data.university_id = UNIVERSITIES[i].to_string();
                            self.update_capital();
                        }
                    }
                }
                7 => {
                    let a = self.options_area(screen_w, screen_h);
                    let bw = a.w * 0.45; let bh = 44.0;
                    let gap = a.w - bw * 2.0;
                    let col_gap = gap / 3.0;
                    for i in 0..4 {
                        let r = Rect { x: a.x + i as f32 * (bw + col_gap), y: a.y + a.h / 2.0 - bh / 2.0, w: bw, h: bh };
                        if r.contains(mouse_x, mouse_y) {
                            self.data.specialty = SPECIALTIES[i].to_string();
                            self.update_capital();
                        }
                    }
                }
                8 => {
                    let a = self.options_area(screen_w, screen_h);
                    let bw = a.w * 0.22; let bh = 44.0;
                    let gap = (a.w - 4.0 * bw) / 3.0;
                    let cy = a.y + a.h / 2.0 - bh / 2.0 + 50.0;
                    for i in 0..4 {
                        let r = Rect { x: a.x + i as f32 * (bw + gap), y: cy, w: bw, h: bh };
                        if r.contains(mouse_x, mouse_y) {
                            self.data.start_region = REGIONS[i].to_string();
                            self.data.start_pos = REGION_POS[i];
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }

    pub fn render(&self, batch: &mut DrawBatch, ui: &crate::graphics::UiRenderer, sw: f32, sh: f32) {
        batch.push_rect(Rect { x: 0.0, y: 0.0, w: sw, h: sh }, Color::new(0.06, 0.06, 0.1, 1.0), 0.0);

        let p = self.panel(sw, sh);
        Panel::new(p.x, p.y, p.w, p.h, Color::new(0.12, 0.12, 0.16, 0.95)).with_corner_radius(8.0).render(batch);

        let title = STEP_TITLES[(self.current_step - 1) as usize];
        let tw = ui.measure_text_width(title);
        ui.push_text(batch, title, p.x + (p.w - tw) / 2.0, p.y + 18.0, FS_TITLE, Color::WHITE);

        let st = format!("Шаг {} / {}", self.current_step, self.total_steps);
        let sw2 = ui.measure_text_width(&st);
        ui.push_text(batch, &st, p.x + (p.w - sw2) / 2.0, p.y + 42.0, FS_LABEL, Color::new(0.45, 0.45, 0.5, 1.0));

        let pv = self.preview_rect(sw, sh);
        Panel::new(pv.x, pv.y, pv.w, pv.h, Color::new(0.08, 0.08, 0.12, 1.0)).with_corner_radius(4.0).render(batch);
        self.render_preview(batch, ui, pv);

        let a = self.options_area(sw, sh);
        let s = (self.current_step - 1) as usize;
        match s {
            0 => self.render_gender(batch, ui, a),
            1 => self.render_height(batch, ui, a),
            2 => self.render_color_swatches(batch, ui, a, &SKIN_COLORS, &["Светлая", "Средняя", "Смуглая", "Тёмная", "Очень тёмная"]),
            3 => self.render_placeholder(batch, ui, a, "Лицо", "Будет доступно в полной версии"),
            4 => self.render_placeholder(batch, ui, a, "Причёска", "Будет доступно в полной версии"),
            5 => self.render_color_swatches(batch, ui, a, &HAIR_COLORS, &["Чёрные", "Каштан", "Русые", "Блонд", "Рыжие"]),
            6 => self.render_options(batch, ui, a, &UNIVERSITIES),
            7 => self.render_options(batch, ui, a, &SPECIALTIES),
            8 => self.render_region(batch, ui, a),
            9 => self.render_summary(batch, ui, a),
            _ => {}
        }

        let back_c = if matches!(self.hovered, HoverTarget::Back) { Color::new(0.28, 0.28, 0.34, 1.0) } else { Color::new(0.18, 0.18, 0.22, 0.9) };
        let next_c = if matches!(self.hovered, HoverTarget::Next) { Color::new(0.28, 0.28, 0.34, 1.0) } else { Color::new(0.18, 0.18, 0.22, 0.9) };
        batch.push_rect(self.btn_back(sw, sh), back_c, 4.0);
        batch.push_rect(self.btn_next(sw, sh), next_c, 4.0);

        let bt = "НАЗАД";
        let nt = if s == 9 { "НАЧАТЬ" } else { "ДАЛЕЕ" };
        let bw = ui.measure_text_width(bt);
        let nw = ui.measure_text_width(nt);
        let br = self.btn_back(sw, sh);
        let nr = self.btn_next(sw, sh);
        ui.push_text(batch, bt, br.x + (br.w - bw) / 2.0, br.y + (br.h - 20.0) / 2.0 + 6.0, FS_BTN, Color::WHITE);
        ui.push_text(batch, nt, nr.x + (nr.w - nw) / 2.0, nr.y + (nr.h - 20.0) / 2.0 + 6.0, FS_BTN, Color::WHITE);
    }

    fn render_preview(&self, batch: &mut DrawBatch, ui: &crate::graphics::UiRenderer, r: Rect) {
        let cy = r.y + r.h * 0.4;
        let label = if self.data.gender == Gender::Male { "МУЖ" } else { "ЖЕН" };
        let lw = ui.measure_text_width(label);
        ui.push_text(batch, label, r.x + (r.w - lw) / 2.0, cy - 10.0, FS_BTN, Color::new(0.8, 0.8, 0.85, 1.0));

        let skin = SKIN_COLORS[self.data.skin_color as usize];
        let head_r = 20.0;
        batch.push_rect(
            Rect { x: r.x + r.w / 2.0 - head_r, y: cy - 40.0, w: head_r * 2.0, h: head_r * 2.0 },
            Color::new(skin[0], skin[1], skin[2], 1.0),
            10.0,
        );
        batch.push_rect(
            Rect { x: r.x + r.w / 2.0 - 12.0, y: cy - 15.0, w: 24.0, h: 40.0 },
            Color::new(skin[0], skin[1], skin[2], 1.0),
            4.0,
        );

        let hair = self.data.hair_color;
        batch.push_rect(
            Rect { x: r.x + r.w / 2.0 - head_r, y: cy - 45.0, w: head_r * 2.0, h: 10.0 },
            Color::new(hair[0], hair[1], hair[2], 1.0),
            4.0,
        );

        ui.push_text(batch, "Одежда", r.x + 10.0, r.y + r.h - 25.0, FS_HINT, Color::new(0.4, 0.4, 0.45, 1.0));
    }

    fn render_gender(&self, batch: &mut DrawBatch, ui: &crate::graphics::UiRenderer, a: Rect) {
        let bw = a.w * 0.45; let bh = 48.0;
        let gap = a.w - bw * 2.0;
        let cy = a.y + a.h / 2.0 - bh / 2.0;
        for i in 0..2 {
            let r = Rect { x: a.x + i as f32 * (bw + gap), y: cy, w: bw, h: bh };
            let sel = self.selected_idx() == i;
            let hov = matches!(self.hovered, HoverTarget::Option(j) if j == i);
            let bg = if sel { Color::new(0.2, 0.4, 0.5, 1.0) } else if hov { Color::new(0.24, 0.24, 0.3, 1.0) } else { Color::new(0.18, 0.18, 0.22, 0.9) };
            batch.push_rect(r.clone(), bg, 6.0);
            let label = if i == 0 { "МУЖСКОЙ" } else { "ЖЕНСКИЙ" };
            let lw = ui.measure_text_width(label);
            ui.push_text(batch, label, r.x + (r.w - lw) / 2.0, r.y + (r.h - 20.0) / 2.0 + 6.0, FS_BTN, Color::WHITE);
        }
    }

    fn render_height(&self, batch: &mut DrawBatch, ui: &crate::graphics::UiRenderer, a: Rect) {
        let slider_w = a.w * 0.8;
        let sx = a.x + (a.w - slider_w) / 2.0;
        let sy = a.y + a.h / 2.0 - 5.0;
        let sh = 8.0;

        batch.push_rect(Rect { x: sx, y: sy, w: slider_w, h: sh }, Color::new(0.2, 0.2, 0.25, 1.0), 4.0);

        let t = (self.data.height_m - 1.50) / 0.50;
        let fill_w = slider_w * t;
        batch.push_rect(Rect { x: sx, y: sy, w: fill_w, h: sh }, Color::new(0.2, 0.45, 0.55, 1.0), 4.0);

        let knob_x = sx + fill_w - 8.0;
        batch.push_rect(Rect { x: knob_x, y: sy - 6.0, w: 16.0, h: sh + 12.0 }, Color::new(0.85, 0.85, 0.9, 1.0), 8.0);

        let label = format!("{:.2} м", self.data.height_m);
        let lw = ui.measure_text_width(&label);
        ui.push_text(batch, &label, a.x + (a.w - lw) / 2.0, sy - 30.0, 26.0, Color::WHITE);

        let min_lw = ui.measure_text_width("1.50");
        let max_lw = ui.measure_text_width("2.00");
        ui.push_text(batch, "1.50", sx - min_lw / 2.0, sy + 14.0, FS_HINT, Color::new(0.4, 0.4, 0.45, 1.0));
        ui.push_text(batch, "2.00", sx + slider_w - max_lw / 2.0, sy + 14.0, FS_HINT, Color::new(0.4, 0.4, 0.45, 1.0));
    }

    fn render_color_swatches(&self, batch: &mut DrawBatch, ui: &crate::graphics::UiRenderer, a: Rect, colors: &[[f32; 3]], labels: &[&str]) {
        let bw = 64.0; let gap = 12.0;
        let total = 5.0 * bw + 4.0 * gap;
        let sx = a.x + (a.w - total) / 2.0;
        let cy = a.y + a.h / 2.0 - bw / 2.0 - 14.0;
        for i in 0..5 {
            let col = colors[i];
            let r = Rect { x: sx + i as f32 * (bw + gap), y: cy, w: bw, h: bw };
            let sel = self.selected_idx() == i;
            batch.push_rect(r.clone(), Color::new(col[0], col[1], col[2], 1.0), 6.0);
            if sel {
                let br = Rect { x: r.x - 3.0, y: r.y - 3.0, w: r.w + 6.0, h: r.h + 6.0 };
                batch.push_rect(br, Color::new(0.9, 0.75, 0.15, 1.0), 8.0);
            }
            let lw = ui.measure_text_width(labels[i]);
            ui.push_text(batch, labels[i], r.x + (r.w - lw) / 2.0, r.y + r.h + 6.0, FS_HINT, Color::new(0.5, 0.5, 0.55, 1.0));
        }
    }

    fn render_placeholder(&self, batch: &mut DrawBatch, ui: &crate::graphics::UiRenderer, a: Rect, title: &str, hint: &str) {
        let tw = ui.measure_text_width(title);
        ui.push_text(batch, title, a.x + (a.w - tw) / 2.0, a.y + a.h / 2.0 - 15.0, FS_BTN, Color::new(0.6, 0.6, 0.65, 1.0));
        let hw = ui.measure_text_width(hint);
        ui.push_text(batch, hint, a.x + (a.w - hw) / 2.0, a.y + a.h / 2.0 + 15.0, FS_HINT, Color::new(0.35, 0.35, 0.4, 1.0));
    }

    fn render_options(&self, batch: &mut DrawBatch, ui: &crate::graphics::UiRenderer, a: Rect, options: &[&str]) {
        let cols = options.len();
        let bw = a.w * 0.45; let bh = 44.0;
        let gap = a.w - bw * 2.0;
        let col_gap = gap / 3.0;
        let total = cols as f32 * bw + (cols as f32 - 1.0) * col_gap;
        let sx = a.x + (a.w - total) / 2.0;
        let cy = a.y + a.h / 2.0 - bh / 2.0;
        for i in 0..cols {
            let r = Rect { x: sx + i as f32 * (bw + col_gap), y: cy, w: bw, h: bh };
            let sel = self.selected_idx() == i;
            let hov = matches!(self.hovered, HoverTarget::Option(j) if j == i);
            let bg = if sel { Color::new(0.2, 0.4, 0.5, 1.0) } else if hov { Color::new(0.24, 0.24, 0.3, 1.0) } else { Color::new(0.18, 0.18, 0.22, 0.9) };
            batch.push_rect(r.clone(), bg, 6.0);
            let lw = ui.measure_text_width(options[i]);
            ui.push_text(batch, options[i], r.x + (r.w - lw) / 2.0, r.y + (r.h - 20.0) / 2.0 + 6.0, FS_BTN, Color::WHITE);
        }
    }

    fn render_region(&self, batch: &mut DrawBatch, ui: &crate::graphics::UiRenderer, a: Rect) {
        let map_h = a.h * 0.55;
        let my = a.y + 10.0;
        Panel::new(a.x, my, a.w, map_h, Color::new(0.1, 0.12, 0.15, 1.0)).with_corner_radius(4.0).render(batch);

        let cx = a.x + a.w / 2.0;
        let ccy = my + map_h / 2.0;
        let positions = [(cx - 30.0, ccy - 20.0), (cx + 80.0, ccy - 10.0), (cx - 60.0, ccy + 30.0), (cx - 120.0, ccy + 10.0)];
        let dot_names = ["Центр", "Промзона", "Спальный", "Пригород"];

        for i in 0..4 {
            let (dx, dy) = positions[i];
            let sel = self.selected_idx() == i;
            let r = 10.0;
            batch.push_rect(Rect { x: dx - r, y: dy - r, w: r * 2.0, h: r * 2.0 }, Color::new(0.3, 0.3, 0.35, 1.0), r as f32);
            if sel {
                batch.push_rect(Rect { x: dx - r - 3.0, y: dy - r - 3.0, w: r * 2.0 + 6.0, h: r * 2.0 + 6.0 }, Color::new(0.9, 0.75, 0.15, 1.0), (r + 3.0) as f32);
            }
            batch.push_rect(Rect { x: dx - 4.0, y: dy - 4.0, w: 8.0, h: 8.0 }, Color::new(0.7, 0.7, 0.75, 1.0), 4.0);
            if sel {
                batch.push_rect(Rect { x: dx - 4.0, y: dy - 4.0, w: 8.0, h: 8.0 }, Color::WHITE, 4.0);
            }
            let lw = ui.measure_text_width(dot_names[i]);
            ui.push_text(batch, dot_names[i], dx - lw / 2.0, dy + r + 6.0, FS_HINT, Color::new(0.55, 0.55, 0.6, 1.0));
        }

        let bw = a.w * 0.22; let bh = 40.0;
        let gap = (a.w - 4.0 * bw) / 3.0;
        let by = my + map_h + 15.0;
        for i in 0..4 {
            let r = Rect { x: a.x + i as f32 * (bw + gap), y: by, w: bw, h: bh };
            let sel = self.selected_idx() == i;
            let hov = matches!(self.hovered, HoverTarget::Option(j) if j == i);
            let bg = if sel { Color::new(0.2, 0.4, 0.5, 1.0) } else if hov { Color::new(0.24, 0.24, 0.3, 1.0) } else { Color::new(0.18, 0.18, 0.22, 0.9) };
            batch.push_rect(r.clone(), bg, 4.0);
            let lw = ui.measure_text_width(REGIONS[i]);
            ui.push_text(batch, REGIONS[i], r.x + (r.w - lw) / 2.0, r.y + (r.h - 18.0) / 2.0 + 5.0, FS_LABEL, Color::WHITE);
        }
    }

    fn render_summary(&self, batch: &mut DrawBatch, ui: &crate::graphics::UiRenderer, a: Rect) {
        let lines = [
            format!("Пол: {}", if self.data.gender == Gender::Male { "Мужской" } else { "Женский" }),
            format!("Рост: {:.2} м", self.data.height_m),
            format!("Вуз: {}", if self.data.university_id.is_empty() { "—" } else { &self.data.university_id }),
            format!("Специальность: {}", if self.data.specialty.is_empty() { "—" } else { &self.data.specialty }),
            format!("Стартовый капитал: {:.0} ₽", self.data.start_capital),
            format!("Регион: {}", if self.data.start_region.is_empty() { "—" } else { &self.data.start_region }),
        ];
        let sy = a.y + 20.0;
        for (i, line) in lines.iter().enumerate() {
            let lw = ui.measure_text_width(line);
            ui.push_text(batch, line, a.x + (a.w - lw) / 2.0, sy + i as f32 * 30.0, FS_BTN, Color::new(0.7, 0.7, 0.75, 1.0));
        }
    }
}
