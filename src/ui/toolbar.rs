use crate::app::App;
use dear_imgui_rs::Ui;

pub enum PendingAction {
    SaveGraph(String),
    LoadGraph(String),
    SaveIni(String),
    LoadIni(String),
    NewGraph,
}

pub fn render_menu_bar(ui: &Ui, app: &mut App) {
    if let Some(_mb) = ui.begin_main_menu_bar() {
        if let Some(_m) = ui.begin_menu("File") {
            if ui.menu_item("New") {
                app.ui.pending = Some(PendingAction::NewGraph);
            }
            if ui.menu_item("Save Graph...") {
                app.ui.pending = Some(PendingAction::SaveGraph("graph.json".into()));
            }
            if ui.menu_item("Load Graph...") {
                app.ui.pending = Some(PendingAction::LoadGraph("graph.json".into()));
            }
            ui.separator();
            if ui.menu_item("Save Layout") {
                app.ui.pending = Some(PendingAction::SaveIni("layout.ini".into()));
            }
            if ui.menu_item("Load Layout") {
                app.ui.pending = Some(PendingAction::LoadIni("layout.ini".into()));
            }
            ui.separator();
            if ui.menu_item("About") {
                app.ui.show_about = true;
            }
        }
        if let Some(_m) = ui.begin_menu("View")
            && ui.menu_item("ImGui Demo Window")
        {
            app.ui.show_demo = !app.ui.show_demo;
        }
    }

    // About popup
    if app.ui.show_about {
        ui.window("About")
            .opened(&mut app.ui.show_about)
            .size([320.0, 160.0], dear_imgui_rs::Condition::Always)
            .build(|| {
                ui.text("Node Graph Editor");
                ui.text("Built with dear-app + dear-imnodes");
            });
    }

    // ImGui demo window (debug aid)
    if app.ui.show_demo {
        ui.show_demo_window(&mut app.ui.show_demo);
    }
}
