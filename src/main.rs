mod app;
mod error;
mod graph;
mod theme;
mod ui;

use app::App;
use dear_app::{AddOnsConfig, AppBuilder, DockingConfig, RunnerConfig};
use error::AppError;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::info;

fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt()
        .with_env_filter("imgui_tutorial=debug")
        .init();

    info!("Node Graph Editor starting...");

    let config = RunnerConfig {
        window_title: "Node Graph Editor".to_owned(),
        window_size: (1280.0, 800.0),
        // Docking is disabled: we render the node graph as a single borderless
        // fullscreen window (no dockspace host, no dock tab/title bar).
        docking: DockingConfig {
            enable: false,
            ..Default::default()
        },
        ..Default::default()
    };

    // NOTE: The tutorial stashes `app` in a plain `Option<App>` shared between
    // `on_setup` and `on_frame` closures. This does NOT compile: both closures
    // are `'static`, so `on_setup` can't borrow `app` while `on_frame` moves it.
    // The fix is interior mutability: `Rc<RefCell<Option<App>>>` lets both
    // closures own a cloned handle to the shared slot.
    let app: Rc<RefCell<Option<App>>> = Rc::new(RefCell::new(None));

    AppBuilder::new()
        .with_config(config)
        .with_addons(AddOnsConfig::auto())
        .on_setup({
            let app = Rc::clone(&app);
            move |ctx| {
                *app.borrow_mut() = Some(App::new(ctx));
            }
        })
        // Load a modern, readable font (Inter) and replace the default ProggyClean
        // bitmap font, which looks pixelated. `rasterizer_density(2.0)` rasterizes
        // glyphs at 2x resolution without changing on-screen size, so text stays
        // crisp on HiDPI (Retina) displays without needing to know the exact scale.
        .on_fonts(|ctx| {
            let font_data = include_bytes!("../assets/Inter-Regular.ttf");
            let mut atlas = ctx.fonts();
            atlas.clear_fonts();
            let cfg = dear_imgui_rs::FontConfig::new().rasterizer_density(2.0);
            atlas.add_font_from_memory_ttf(font_data, 18.0, Some(&cfg), None);
        })
        // NOTE: The tutorial's on_frame uses `move |ui|` (one arg), but the actual
        // dear-app 0.15.1 API is `FnMut(&Ui, &mut AddOns)` — two arguments.
        .on_frame({
            let app = Rc::clone(&app);
            move |ui, _addons| {
                if let Some(a) = app.borrow_mut().as_mut() {
                    crate::ui::render(ui, a);
                }
            }
        })
        .run()
        .map_err(|e| AppError::Init(e.to_string()))?;

    info!("Shutting down.");
    Ok(())
}
