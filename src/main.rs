use rtgc::app::App;
use rtgc::audio::audio_manager::RtgcAudioManager;
use rtgc::platform::paths::AppPaths;
use rtgc::platform::window;
use anyhow::Result;
use std::sync::Arc;
use winit::event_loop::EventLoop;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("rtgc=debug,warn")
        .init();

    tracing::info!("RTGC-1.0 запускается...");

    let paths = AppPaths::resolve()?;
    paths.ensure_directories()?;

    let event_loop = EventLoop::new()?;

    let (window, gl_surface, gl_context) = window::create_window(
        &event_loop,
        1280,
        720,
        "RTGC 1.0",
    )?;

    let _window = window;

    let audio = RtgcAudioManager::new()?;

    let mut app = App::new(
        Arc::new(gl_context),
        gl_surface,
        audio,
        paths,
        1280,
        720,
    )?;

    event_loop.run(move |event, _target| {
        app.handle_event(&event);
    })?;

    Ok(())
}
