//? The lib target has no entry point so the compiler sees everything as unused.
#![allow(dead_code)]
use engine::{
    AnimationDef, AnimationState, AudioTrack, Context, FixedTime, GameAction, Key, Rect,
    StaticSoundData, Vec2, egui, load_sound_data,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DinoAction {
    Blink,
}

impl GameAction for DinoAction {
    fn count() -> usize { 1 }
    fn index(&self) -> usize { 0 }
    fn from_index(i: usize) -> Option<Self> { if i == 0 { Some(Self::Blink) } else { None } }
}

const RES_W: u32 = 360;
const RES_H: u32 = 180;
const BG_W: f32 = 288.0;
const BG_H: f32 = 180.0;
const SPRITE_SZ: f32 = 32.0; //* source frame size in sheet
const SHEET_COLS: usize = 10;
const GROUND_H: f32 = 10.0;
const GROUND_Y: f32 = RES_H as f32 - GROUND_H; //* 170.0
const GROUND_COL: [f32; 4] = [0.16, 0.32, 0.17, 1.0]; //* #2a532b
const GROUND_TOP_COL: [f32; 4] = [0.58, 0.78, 0.35, 1.0]; //* #95c85a
const DINO_X: f32 = 32.0;
const DINO_W: f32 = 32.0;
const DINO_H: f32 = 32.0;
const DINO_TOP: f32 = GROUND_Y - DINO_H + 3.0;
const DINO_SINK: f32 = 6.0;
const MIN_BLINK_DUR: f32 = 0.30; //* instant tap
const MAX_BLINK_DUR: f32 = 1.10; //* full hold
const MAX_HOLD: f32 = 0.65; //* hold time to reach MAX_BLINK_DUR
const BASE_SPD: f32 = 120.0;
const SPD_INC: f32 = 10.0;
const OBS_MIN_W: f32 = 7.0;
const OBS_MAX_W: f32 = 20.0;
const OBS_MIN_H: f32 = 26.0;
const OBS_MAX_H: f32 = 54.0;
const CACTUS_BODY: [f32; 4] = [0.18, 0.55, 0.20, 1.0];
const CACTUS_DARK: [f32; 4] = [0.09, 0.33, 0.11, 1.0];
const CACTUS_TIP: [f32; 4] = [0.35, 0.78, 0.28, 1.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const AQUA32: egui::Color32 = egui::Color32::from_rgb(127, 255, 212);
const RED32: egui::Color32 = egui::Color32::from_rgb(229, 80, 57); //* rgb(229, 80, 57)
const GOLD32: egui::Color32 = egui::Color32::from_rgb(250, 152, 58);
const DIM_WHITE: egui::Color32 = egui::Color32::from_rgba_premultiplied(255, 255, 255, 80);
const PANEL_BG: egui::Color32 = egui::Color32::from_rgba_premultiplied(5, 10, 18, 200);
const PANEL_STROKE: egui::Color32 = egui::Color32::from_rgba_premultiplied(127, 255, 212, 60);

fn game_speed(score: u32) -> f32 { BASE_SPD + (score / 6).min(100) as f32 * SPD_INC }

fn lcg(seed: &mut u64) -> f32 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*seed >> 33) as f32) / (u32::MAX as f32)
}

fn frame_rect(frame: usize) -> Rect {
    Rect::new((frame % SHEET_COLS) as f32 * SPRITE_SZ, (frame / SHEET_COLS) as f32 * SPRITE_SZ, SPRITE_SZ, SPRITE_SZ)
}

fn play_sfx(snd: &Option<StaticSoundData>, audio: &mut engine::AudioManager) {
    if let Some(d) = snd { audio.play_oneshot(d, AudioTrack::Sfx); }
}
fn play_loop_snd(snd: &Option<StaticSoundData>, audio: &mut engine::AudioManager) {
    if let Some(d) = snd { audio.play_loop_sfx(d); }
}

fn draw_title(ui: &mut egui::Ui, text: &str, size: f32, fg: egui::Color32, outline: egui::Color32, w: f32) {
    let font = egui::FontId::monospace(size);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 40.0), egui::Sense::hover());
    let c = rect.center();
    for dx in -2i32..=2 { for dy in -2i32..=2 {
        if dx != 0 || dy != 0 {
            ui.painter().text(c + egui::vec2(dx as f32, dy as f32), egui::Align2::CENTER_CENTER, text, font.clone(), outline);
        }
    }}
    ui.painter().text(c, egui::Align2::CENTER_CENTER, text, font, fg);
}

#[derive(PartialEq)]
enum State { Idle, Playing, Dead }

struct DinoBlink {
    state: State,
    blink_timer: f32,
    holding_blink: bool,
    hold_duration: f32,
    obstacles: Vec<(f32, f32, f32)>, //* (x, height, width)
    ticks: u64,
    hi_ticks: u64,
    spawn_timer: f32,
    seed: u64,
    tex_bg0: usize, tex_bg1: usize, tex_bg2: usize,
    tex_white: usize, tex_dino: usize,
    anim: AnimationState,
    bg0_x: f32, bg1_x: f32, bg2_x: f32,
    snd_bg: Option<StaticSoundData>,
    snd_blink: Option<StaticSoundData>,
    snd_death: Option<StaticSoundData>,
    snd_run: Option<StaticSoundData>,
    was_blinking: bool,
}

impl DinoBlink {
    fn blinking(&self) -> bool { self.blink_timer > 0.0 }
    fn score(&self) -> u32 { (self.ticks / 60) as u32 }
    fn hi(&self) -> u32 { (self.hi_ticks / 60) as u32 }

    fn reset(&mut self) {
        self.state = State::Playing;
        self.blink_timer = 0.0;
        self.holding_blink = false;
        self.hold_duration = 0.0;
        self.obstacles.clear();
        self.ticks = 0;
        self.spawn_timer = 1.2;
        self.bg0_x = 0.0; self.bg1_x = 0.0; self.bg2_x = 0.0;
        self.anim.play("idle");
    }
}

impl engine::GameApp for DinoBlink {
    type Action = DinoAction;

    fn window_title() -> &'static str { "dino blink" }
    fn internal_resolution() -> (u32, u32) { (RES_W, RES_H) }
    fn window_icon() -> Option<&'static [u8]> { Some(include_bytes!("../assets/sprites/icon.png")) }

    fn init(ctx: &mut Context<DinoAction>) -> Self {
        ctx.input.input_map_mut().bind_key(Key::Space, DinoAction::Blink);

        let tex_bg0  = ctx.load_texture(include_bytes!("../assets/sprites/bg0.png"), "bg0");
        let tex_bg1  = ctx.load_texture(include_bytes!("../assets/sprites/bg1.png"), "bg1");
        let tex_bg2  = ctx.load_texture(include_bytes!("../assets/sprites/bg2.png"), "bg2");
        let tex_white = ctx.load_texture(include_bytes!("../assets/sprites/white.png"), "white");
        let tex_dino  = ctx.load_texture(include_bytes!("../assets/sprites/dino_sheet.png"), "dino");

        let anim = AnimationState::new(vec![
            AnimationDef::new("idle", 0, 6, 0.15, true),
            AnimationDef::new("run", 10, 6, 0.10, true),
        ], "idle");

        let snd_bg    = load_sound_data(include_bytes!("../assets/audio/bg.ogg"));
        let snd_blink = load_sound_data(include_bytes!("../assets/audio/blink.wav"));
        let snd_death = load_sound_data(include_bytes!("../assets/audio/death.wav"));
        let snd_run   = load_sound_data(include_bytes!("../assets/audio/run.wav"));

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(d) = &snd_bg {
            ctx.audio.play_music(d, 1.5);
            ctx.audio.set_music_live_volume(0.25, 0.0);
        }

        Self {
            state: State::Idle,
            blink_timer: 0.0, holding_blink: false, hold_duration: 0.0,
            obstacles: Vec::new(),
            ticks: 0, hi_ticks: 0, spawn_timer: 1.2,
            seed: 0xdeadbeef_cafe1337,
            tex_bg0, tex_bg1, tex_bg2, tex_white, tex_dino,
            anim,
            bg0_x: 0.0, bg1_x: 0.0, bg2_x: 0.0,
            snd_bg, snd_blink, snd_death, snd_run,
            was_blinking: false,
        }
    }

    fn fixed_update(&mut self, ctx: &mut Context<DinoAction>, fixed_time: &FixedTime) {
        if self.state != State::Playing {
            return;
        }

        let dt = fixed_time.fixed_dt;
        let spd = game_speed(self.score());

        for o in &mut self.obstacles {
            o.0 -= spd * dt;
        }
        self.obstacles.retain(|o| o.0 > -30.0);

        self.bg0_x += spd * 0.03 * dt;
        self.bg1_x += spd * 0.10 * dt;
        self.bg2_x += spd * 0.25 * dt;

        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            let h = OBS_MIN_H + lcg(&mut self.seed) * (OBS_MAX_H - OBS_MIN_H);
            let w = OBS_MIN_W + lcg(&mut self.seed) * (OBS_MAX_W - OBS_MIN_W);
            self.obstacles.push((RES_W as f32 + 20.0, h, w));
            self.spawn_timer = 0.5 + lcg(&mut self.seed) * 1.1;
        }

        self.ticks += 1;

        if !self.blinking() {
            let (dx, dw) = (DINO_X + 4.0, DINO_W - 8.0);
            for o in &self.obstacles {
                if o.0 < dx + dw && o.0 + o.2 > dx {
                    self.hi_ticks = self.hi_ticks.max(self.ticks);
                    self.state = State::Dead;
                    play_sfx(&self.snd_death, &mut ctx.audio);
                    ctx.audio.stop_loop_sfx(0.1);
                    ctx.audio.set_music_live_volume(0.18, 0.08);
                    self.was_blinking = false;
                    ctx.trigger_shake(5.0, 0.25);
                    ctx.trigger_freeze(4);
                    return;
                }
            }
        }
    }

    fn update(&mut self, ctx: &mut Context<DinoAction>) {
        ctx.set_fullscreen_enabled(false);
        if ctx.input.any_keyboard_or_mouse() { ctx.audio.notify_user_gesture(); }
        self.blink_timer = (self.blink_timer - ctx.delta_time).max(0.0);

        let held = ctx.input.is_action_pressed(DinoAction::Blink);
        let just_pressed = ctx.input.is_action_just_pressed(DinoAction::Blink);

        if just_pressed {
            match self.state {
                State::Idle | State::Dead => {
                    if !ctx.audio.has_active_music() {
                        if let Some(d) = &self.snd_bg { ctx.audio.play_music(d, 0.5); }
                    }
                    ctx.audio.set_music_live_volume(1.0, 0.3);
                    self.reset();
                    self.anim.play("run");
                    play_loop_snd(&self.snd_run, &mut ctx.audio);
                }
                State::Playing if !self.blinking() => {
                    self.holding_blink = true;
                    self.hold_duration = 0.0;
                }
                _ => {}
            }
        }

        if self.state == State::Playing && self.holding_blink {
            if held {
                self.hold_duration = (self.hold_duration + ctx.delta_time).min(MAX_HOLD);
            } else {
                //? Blink scaled by hold time.
                let t = (self.hold_duration / MAX_HOLD).clamp(0.0, 1.0);
                self.blink_timer = MIN_BLINK_DUR + t * (MAX_BLINK_DUR - MIN_BLINK_DUR);
                self.holding_blink = false;
                self.hold_duration = 0.0;
                play_sfx(&self.snd_blink, &mut ctx.audio);
            }
        }

        if self.state == State::Playing {
            self.anim.update(ctx.delta_time);
            let now_blinking = self.blinking();
            if now_blinking && !self.was_blinking { ctx.audio.stop_loop_sfx(0.05); }
            else if !now_blinking && self.was_blinking { play_loop_snd(&self.snd_run, &mut ctx.audio); }
            self.was_blinking = now_blinking;
        }
    }

    fn render(&mut self, ctx: &mut Context<DinoAction>) {
        let bg_src = Rect::new(0.0, 0.0, BG_W, BG_H);
        let num_bg = (RES_W as f32 / BG_W) as i32 + 2;

        for &(scroll, tex) in &[(self.bg0_x, self.tex_bg0), (self.bg1_x, self.tex_bg1), (self.bg2_x, self.tex_bg2)] {
            let off = scroll % BG_W;
            for i in 0..num_bg {
                ctx.draw_sprite_from_sheet(Vec2::new(i as f32 * BG_W - off, 0.0), Vec2::new(BG_W, BG_H), WHITE, bg_src, false, tex);
            }
        }

        let w1 = Rect::new(0.0, 0.0, 1.0, 1.0);
        let sw = RES_W as f32;

        ctx.draw_sprite_from_sheet(Vec2::new(0.0, GROUND_Y), Vec2::new(sw, GROUND_H), GROUND_COL, w1, false, self.tex_white);
        ctx.draw_sprite_from_sheet(Vec2::new(0.0, GROUND_Y), Vec2::new(sw, 2.0), GROUND_TOP_COL, w1, false, self.tex_white);

        let tw = self.tex_white;
        let obstacles: Vec<(f32, f32, f32)> = self.obstacles.clone();
        for &(x, h, ow) in &obstacles {
            let top = GROUND_Y - h;
            let (arm_y1, arm_y2) = (top + h * 0.35, top + h * 0.62);
            let parts: &[(Vec2, Vec2, [f32; 4])] = &[
                (Vec2::new(x,       top),    Vec2::new(ow,       h),   CACTUS_BODY),
                (Vec2::new(x,       top),    Vec2::new(3.0,      h),   CACTUS_DARK),
                (Vec2::new(x + 2.0, top),    Vec2::new(ow - 4.0, 4.0), CACTUS_TIP),
                (Vec2::new(x - 6.0, arm_y1), Vec2::new(6.0,     5.0), CACTUS_BODY),
                (Vec2::new(x + ow,  arm_y2), Vec2::new(5.0,     5.0), CACTUS_BODY),
            ];
            for &(pos, sz, col) in parts { ctx.draw_sprite_from_sheet(pos, sz, col, w1, false, tw); }
        }

        if let Some((anim_def, frame_idx)) = self.anim.current() {
            let frame = anim_def.start_frame + frame_idx;
            let alpha = if self.blinking() { 0.15 } else { 1.0 };
            ctx.draw_sprite_from_sheet(Vec2::new(DINO_X, DINO_TOP + DINO_SINK), Vec2::new(DINO_W, DINO_H), [1.0, 1.0, 1.0, alpha], frame_rect(frame), false, self.tex_dino);
        }
    }

    fn ui(
        &mut self,
        egui_ctx: &egui::Context,
        _ctx: &mut Context<DinoAction>,
        scene: &mut engine::SceneParams,
    ) {
        scene.background_color = [0.207, 0.282, 0.274]; //* #354b46
        scene.fog_enabled = false;

        let md = egui::FontId::monospace(12.0);
        let lg = egui::FontId::monospace(16.0);

        let pill = egui::Frame {
            fill: PANEL_BG,
            stroke: egui::Stroke::new(1.0, PANEL_STROKE),
            inner_margin: egui::Margin::same(5),
            outer_margin: egui::Margin::ZERO,
            corner_radius: egui::CornerRadius::same(2),
            shadow: egui::Shadow::NONE,
        };

        match self.state {
            State::Idle => {
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(egui_ctx, |ui| {
                        ui.painter().rect_filled(
                            ui.max_rect(),
                            0.0,
                            egui::Color32::from_rgba_premultiplied(4, 10, 8, 200),
                        );
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.add_space(40.0);
                        draw_title(ui, "DINO BLINK", 28.0, AQUA32, egui::Color32::from_rgba_premultiplied(0, 40, 28, 255), 200.0);
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("blink through chaos like a warlock")
                                    .font(md.clone())
                                    .color(DIM_WHITE),
                            );
                            ui.add_space(32.0);
                            ui.allocate_ui(egui::vec2(260.0, 40.0), |ui| {
                                pill.show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.label(
                                            egui::RichText::new("Press SPACE to run!")
                                                .font(lg.clone())
                                                .color(GOLD32),
                                        );
                                    });
                                });
                            });
                            if self.hi() > 0 {
                                ui.add_space(16.0);
                                ui.label(
                                    egui::RichText::new(format!("best  {:05}", self.hi()))
                                        .font(md.clone())
                                        .color(GOLD32),
                                );
                            }
                        });
                    });
            }

            State::Playing => {
                egui::Area::new(egui::Id::new("hud_score"))
                    .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
                    .show(egui_ctx, |ui| {
                        pill.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 6.0;
                                ui.label(
                                    egui::RichText::new(format!("{:05}", self.score()))
                                        .font(md.clone())
                                        .color(egui::Color32::WHITE),
                                );
                                if self.hi() > 0 {
                                    ui.label(
                                        egui::RichText::new(format!("HI {:05}", self.hi()))
                                            .font(md.clone())
                                            .color(GOLD32),
                                    );
                                }
                            });
                        });
                    });

                if self.holding_blink {
                    let charge = (self.hold_duration / MAX_HOLD).clamp(0.0, 1.0);
                    let filled = (charge * 16.0).round() as usize;
                    let bar: String = (0..16)
                        .map(|i| if i < filled { '█' } else { '░' })
                        .collect();
                    let label = if charge >= 1.0 { "CHARGED" } else { "charging" };
                    let col = if charge >= 1.0 { AQUA32 } else { GOLD32 };
                    egui::Area::new(egui::Id::new("blink_bar"))
                        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 50.0))
                        .show(egui_ctx, |ui| {
                            pill.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 4.0;
                                    ui.label(
                                        egui::RichText::new("BLINK").font(md.clone()).color(col),
                                    );
                                    ui.label(egui::RichText::new(&bar).font(md.clone()).color(col));
                                    ui.label(
                                        egui::RichText::new(label).font(md.clone()).color(col),
                                    );
                                });
                            });
                        });
                }
            }

            State::Dead => {
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(egui_ctx, |ui| {
                        ui.painter().rect_filled(
                            ui.max_rect(), 0.0,
                            egui::Color32::from_rgba_premultiplied(2, 4, 8, 185),
                        );
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.add_space(30.0);
                            draw_title(ui, "Dino Perished :(", 28.0, RED32, egui::Color32::from_rgba_premultiplied(80, 0, 0, 255), 240.0);
                            ui.add_space(20.0);
                            
                            ui.allocate_ui(egui::vec2(150.0, 70.0), |ui| {
                                pill.show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        ui.spacing_mut().item_spacing.y = 4.0;
                                        let row = |ui: &mut egui::Ui, label: &str, val: String, col: egui::Color32| {
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 0.0;
                                                ui.label(egui::RichText::new(label).font(md.clone()).color(DIM_WHITE));
                                                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                                                    ui.label(egui::RichText::new(val).font(md.clone()).color(col));
                                                });
                                            });
                                        };
                                        row(ui, "Score:", format!("{:05}", self.score()), egui::Color32::WHITE);
                                        row(ui, "Best:", format!("{:05}", self.hi()), GOLD32);
                                    });
                                });
                            });
                            
                            ui.add_space(24.0);
                            
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new("Try Again?")
                                        .font(egui::FontId::monospace(14.0)).color(DIM_WHITE),
                                );
                                ui.label(
                                    egui::RichText::new("Press SPACE to restart")
                                        .font(md.clone()).color(GOLD32),
                                );
                            });
                        });
                    });
            }
        }
    }
}

pub fn run_game() {
    engine::run::<DinoBlink>();
}

#[cfg(target_arch = "wasm32")]
mod wasm_entry {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen(start)]
    pub fn wasm_main() { engine::run_wasm::<super::DinoBlink>(); }
}
