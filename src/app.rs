use crate::audio::audio_manager::RtgcAudioManager;
use crate::core::app_state::AppState;
use crate::core::timer::FrameTimer;
use crate::graphics::rhi::opengl::device::GlDevice;
use crate::graphics::ui_renderer::batch::{Color, DrawBatch};
use crate::graphics::UiRenderer;
use crate::platform::input::InputState;
use crate::platform::paths::AppPaths;
use crate::platform::window::GlSurface;
use crate::graphics::CommandBuffer;
use crate::screens::character_creation::character_creation_screen::CharacterCreationScreen;
use crate::screens::loading::loading_screen::LoadingScreen;
use crate::screens::main_menu::main_menu_screen::MainMenuScreen;
use crate::screens::settings::settings_screen::SettingsScreen;
use crate::screens::splash::splash_screen::SplashScreen;
use anyhow::Result;
use std::sync::Arc;
use glow::HasContext;
use winit::event::WindowEvent;

pub struct App {
    pub state: AppState,
    pub timer: FrameTimer,
    pub input: InputState,
    pub paths: AppPaths,
    pub gl_device: GlDevice,
    pub gl_surface: GlSurface,
    pub gl_context: Arc<glow::Context>,
    pub audio: RtgcAudioManager,
    pub draw_batch: DrawBatch,
    pub command_buffer: CommandBuffer,
    pub ui_renderer: UiRenderer,

    splash: SplashScreen,
    main_menu: Option<MainMenuScreen>,
    settings: Option<SettingsScreen>,
    character_creation: Option<CharacterCreationScreen>,
    loading: Option<LoadingScreen>,

    screen_width: f32,
    screen_height: f32,
    pub should_exit: bool,
}

impl App {
    pub fn new(
        gl_context: Arc<glow::Context>,
        gl_surface: GlSurface,
        audio: RtgcAudioManager,
        paths: AppPaths,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let gl_device = GlDevice::new(gl_context.clone());
        let ui_renderer = UiRenderer::new(gl_context.clone())
            .expect("Failed to create UI renderer");

        Ok(Self {
            state: AppState::Splash,
            timer: FrameTimer::new(),
            input: InputState::new(),
            paths,
            gl_device,
            gl_surface,
            gl_context,
            audio,
            draw_batch: DrawBatch::new(),
            command_buffer: CommandBuffer::new(),
            ui_renderer,
            splash: SplashScreen::new(),
            main_menu: None,
            settings: None,
            character_creation: None,
            loading: None,
            screen_width: width as f32,
            screen_height: height as f32,
            should_exit: false,
        })
    }

    pub fn handle_event(&mut self, event: &winit::event::Event<()>) {
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    self.should_exit = true;
                }
                WindowEvent::Resized(size) => {
                    self.screen_width = size.width as f32;
                    self.screen_height = size.height as f32;
                    unsafe {
                        self.gl_context.viewport(0, 0, size.width as i32, size.height as i32);
                    }
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    use winit::keyboard::Key;
                    use winit::keyboard::NamedKey;
                    if let Key::Named(NamedKey::Escape) = &event.logical_key {
                        match &self.state {
                            AppState::Playing => { self.state = AppState::PauseMenu; }
                            AppState::PauseMenu => { self.state = AppState::Playing; }
                            AppState::Settings { return_to } => { self.state = *return_to.clone(); }
                            AppState::CharacterCreation => { self.state = AppState::MainMenu; }
                            _ => {}
                        }
                    }
                }
                other => {
                    self.input.handle_window_event(other);
                }
            },
            winit::event::Event::AboutToWait => {
                self.update();
                self.render();
            }
            _ => {}
        }
    }

    fn update(&mut self) {
        self.timer.tick();

        let mouse_x = self.input.mouse_pos.x;
        let mouse_y = self.input.mouse_pos.y;
        let mouse_just_pressed = self.input.is_mouse_button_just_pressed(0);
        let mouse_held = self.input.is_mouse_button_pressed(0);
        let dt = self.timer.dt as f32;

        let next_state: Option<AppState> = match &mut self.state {
            AppState::Splash => self.splash.update(dt),
            AppState::MainMenu => {
                self.main_menu.as_mut().and_then(|menu| {
                    menu.update(mouse_x, mouse_y, mouse_just_pressed, dt, self.screen_width, self.screen_height)
                })
            }
            AppState::Settings { .. } => {
                self.settings.as_mut().and_then(|s| s.update(mouse_x, mouse_y, mouse_just_pressed))
            }
            AppState::CharacterCreation => {
                self.character_creation.as_mut().and_then(|s| {
                    s.update(mouse_x, mouse_y, mouse_just_pressed, mouse_held, self.screen_width, self.screen_height)
                })
            }
            AppState::Loading { .. } => {
                self.loading.as_mut().and_then(|l| l.update(dt))
            }
            AppState::SaveSelect => Some(AppState::MainMenu),
            AppState::Playing => None,
            AppState::PauseMenu => None,
        };

        if let Some(next) = next_state {
            self.transition_to(next);
        }

        self.input.clear_frame_end();
    }

    fn transition_to(&mut self, next: AppState) {
        tracing::info!("Transitioning to {:?}", next);
        match &next {
            AppState::MainMenu => { self.main_menu = Some(MainMenuScreen::new()); }
            AppState::Settings { return_to } => {
                self.settings = Some(SettingsScreen::new(*return_to.clone()));
            }
            AppState::CharacterCreation => {
                self.character_creation = Some(CharacterCreationScreen::new());
            }
            AppState::Loading { character_data } => {
                self.loading = Some(LoadingScreen::new(*character_data.clone()));
            }
            _ => {}
        }
        self.state = next;
    }

    fn render(&mut self) {
        let sw = self.screen_width;
        let sh = self.screen_height;

        unsafe {
            self.gl_context.clear_color(0.05, 0.05, 0.08, 1.0);
            self.gl_context.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        self.draw_batch.clear();

        // Background for current state
        match &self.state {
            AppState::Splash => {
                self.splash.render(&mut self.draw_batch, sw, sh);
            }
            AppState::MainMenu => {
                if let Some(ref menu) = self.main_menu {
                    menu.render(&mut self.draw_batch, sw, sh);
                }
            }
            AppState::Settings { .. } => {
                if let Some(ref settings) = self.settings {
                    settings.render(&mut self.draw_batch, sw, sh);
                }
            }
            AppState::CharacterCreation => {
                if let Some(ref screen) = self.character_creation {
                    screen.render(&mut self.draw_batch, &self.ui_renderer, sw, sh);
                }
            }
            AppState::Loading { .. } => {
                if let Some(ref loading) = self.loading {
                    loading.render(&mut self.draw_batch, sw, sh);
                }
            }
            _ => {}
        }

        // Text overlay for all states
        match &self.state {
            AppState::Splash => {
                let text = "RTGC 1.0";
                let text_w = self.ui_renderer.measure_text_width(text);
                let x = (sw - text_w) / 2.0;
                let baseline = sh / 2.0 + 72.0;
                self.ui_renderer.push_text(&mut self.draw_batch, text, x, baseline, 24.0, Color::WHITE);
            }
            AppState::MainMenu => {
                let btn_w = 300.0;
                let btn_h = 50.0;
                let start_y = sh / 2.0 + 30.0;
                let spacing = btn_h + 16.0;
                let center_x = (sw - btn_w) / 2.0;
                let labels = ["НОВАЯ ИГРА", "ЗАГРУЗИТЬ ИГРУ", "НАСТРОЙКИ", "ВЫХОД"];
                for (i, label) in labels.iter().enumerate() {
                    let btn_top = start_y + (i as f32) * spacing;
                    let baseline = btn_top + btn_h / 2.0 + 4.0;
                    let text_w = self.ui_renderer.measure_text_width(label);
                    let x = center_x + (btn_w - text_w) / 2.0;
                    self.ui_renderer.push_text(&mut self.draw_batch, label, x, baseline, 20.0, Color::WHITE);
                }
            }
            AppState::Settings { .. } => {
                let text = "НАСТРОЙКИ";
                let text_w = self.ui_renderer.measure_text_width(text);
                let x = (sw - text_w) / 2.0;
                let baseline = sh / 2.0 - 166.0;
                self.ui_renderer.push_text(&mut self.draw_batch, text, x, baseline, 24.0, Color::WHITE);

                let btn_text = "НАЗАД";
                let btn_w = 200.0;
                let btn_h = 48.0;
                let btn_top = sh - 80.0;
                let baseline = btn_top + btn_h / 2.0 + 4.0;
                let text_w = self.ui_renderer.measure_text_width(btn_text);
                let btn_x = (sw - btn_w) / 2.0 + (btn_w - text_w) / 2.0;
                self.ui_renderer.push_text(&mut self.draw_batch, btn_text, btn_x, baseline, 20.0, Color::WHITE);
            }
            AppState::CharacterCreation => {
            }
            AppState::Loading { .. } => {
                let text = "ЗАГРУЗКА...";
                let text_w = self.ui_renderer.measure_text_width(text);
                let x = (sw - text_w) / 2.0;
                let baseline = sh / 2.0 - 36.0;
                self.ui_renderer.push_text(&mut self.draw_batch, text, x, baseline, 22.0, Color::WHITE);

                if let Some(ref loading) = self.loading {
                    let msg = loading.stage_message();
                    let msg_w = self.ui_renderer.measure_text_width(msg);
                    let msg_x = (sw - msg_w) / 2.0;
                    let msg_baseline = sh / 2.0 + 2.0;
                    self.ui_renderer.push_text(&mut self.draw_batch, msg, msg_x, msg_baseline, 16.0, Color::new(0.5, 0.5, 0.55, 1.0));
                }
            }
            _ => {}
        }

        self.ui_renderer.render(&self.draw_batch, sw, sh);

        use glutin::prelude::GlSurface;
        let _ = self.gl_surface.surface.swap_buffers(&self.gl_surface.context).ok();
    }
}
