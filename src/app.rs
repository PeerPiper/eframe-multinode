mod error;
mod platform;
mod rdx_runner;

use std::collections::BTreeSet;

use egui::ScrollArea;
pub(crate) use platform::Platform;
use platform::Settings;
use rdx::layer::Inner as _;
use rdx_runner::RdxRunner;
use web_time::Instant;

const APP_KEY: &str = concat!("eframe-app-", env!("CARGO_PKG_NAME"));

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
//#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MultinodeApp {
    /// Platform  specific handlers for native and web     
    #[serde(skip)]
    platform: Platform,

    settings: Settings,

    /// Last time we saved the app state
    #[serde(skip, default = "default_last_save")]
    last_save: Instant,

    /// If the backup save has been used or not
    #[serde(skip, default)]
    needs_save: bool,

    /// Open plugins
    open: BTreeSet<String>,
}

impl Default for MultinodeApp {
    fn default() -> Self {
        Self {
            platform: Platform::default(),
            settings: Settings::default(),
            last_save: default_last_save(),
            needs_save: true,
            open: BTreeSet::new(),
        }
    }
}

fn default_last_save() -> Instant {
    Instant::now()
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
        // also call State's Inner::save for each plugin state
        self.platform
            .rdx_runner
            .plugins
            .iter_mut()
            .for_each(|(_name, plugin)| {
                plugin.plugin.lock().unwrap().store().data().save();
            });
        self.last_save = Instant::now();
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(20) // Set autosave interval to 20 seconds
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        if self.needs_save && self.last_save.elapsed().as_secs() >= 30 {
            tracing::info!("💾 Backup save, saving app state");
            self.platform
                .rdx_runner
                .plugins
                .iter_mut()
                .for_each(|(_name, plugin)| {
                    plugin.plugin.lock().unwrap().store().data().save();
                });
            self.last_save = Instant::now();
            self.needs_save = false;
        }

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

        let plugin_names = self
            .platform
            .rdx_runner
            .plugins
            .keys()
            .cloned()
            .collect::<Vec<_>>(); // clone keys to avoid borrowing issues

        egui::SidePanel::left("plugin_panel")
            .resizable(false)
            .default_width(160.0)
            .min_width(160.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.vertical_centered(|ui| {
                    ui.heading("🔌 Plugins");
                });

                ui.separator();

                self.platform.load_plugin(ctx, ui);

                ui.separator();
                ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                        // list plugins here
                        for name in plugin_names {
                            let open = &mut self.open;
                            let mut is_open = open.contains(&name);
                            ui.toggle_value(&mut is_open, name.clone());
                            set_open(open, &name, is_open);
                        }
                    });
                });
            });

        // Right panel is for AI LLM chat box
        #[cfg(not(target_arch = "wasm32"))]
        egui::SidePanel::right("chat_panel")
            .resizable(true)
            .default_width(160.0)
            .min_width(160.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.vertical_centered(|ui| {
                    ui.heading("💬 AI Chat");
                });

                ui.separator();

                // chat box here for ollama chat
                self.platform.chat(ctx, ui);
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

            ui.vertical(|ui| {
                if let Some(addr) = self.platform.addr() {
                    ui.label(format!("Node Address: {:?}", addr));
                    self.settings.show(ctx, ui, &addr);
                }
                self.platform.show(ctx, ui);
            });

            // Show plugins
            let RdxRunner { plugins, .. } = &mut self.platform.rdx_runner;
            for (name, plugin) in plugins.iter_mut() {
                // if is open
                if !self.open.contains(name) {
                    continue;
                }
                plugin.render_rhai(ctx.clone());
            }
        });
    }
}

fn set_open(open: &mut BTreeSet<String>, key: &String, is_open: bool) {
    if is_open {
        if !open.contains(key) {
            open.insert(key.to_owned());
        }
    } else {
        open.remove(key);
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
