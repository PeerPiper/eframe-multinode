mod error;
mod file_dialog;
mod platform;
//mod widgets;

pub(crate) use platform::Platform;
use platform::Settings;

const APP_KEY: &str = concat!("eframe-app-", env!("CARGO_PKG_NAME"));

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MultinodeApp {
    /// Platform  specific handlers for native and web     
    #[serde(skip)]
    platform: Platform,

    file_dialog: file_dialog::FileDialog,

    settings: Settings,
}

impl MultinodeApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        eprintln!("app_key: {}", APP_KEY);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            if let Some(app) = eframe::get_value(storage, APP_KEY) {
                tracing::info!("💾 Loaded app state from disk");
                return app;
            }
        }

        tracing::info!("🆕 ⛔ No app state found on disk");
        Default::default()
    }
}

impl eframe::App for MultinodeApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        tracing::info!("💾 Saving app state");
        eframe::set_value(storage, APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        // pass the ctx to the platform
        if !self.platform.egui_ctx() {
            egui_material_icons::initialize(ctx);
            self.platform.set_egui_ctx(ctx.clone());
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                ui.add(egui::github_link_file!(
                    "https://github.com/PeerPiper/egui-multinode/blob/main/",
                    format!("🖹 Rust Source Code")
                ));
                egui::warn_if_debug_build(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // We needs a wallet widget first, to unlock with username and password.

            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Load Plugin");

            let platform_clone = self.platform.clone();
            let on_load_callback = move |name, bytes| {
                platform_clone.load_plugin(name, bytes);
            };
            if let Err(e) = self.file_dialog.file_dialog(ui, on_load_callback) {
                tracing::error!("Failed to open file dialog: {:?}", e);
            }

            ui.vertical(|ui| {
                if let Some(addr) = self.platform.addr() {
                    ui.label(format!("Node Address: {:?}", addr));
                    self.settings.show(ctx, ui, &addr);
                }
                self.platform.show(ctx, ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
