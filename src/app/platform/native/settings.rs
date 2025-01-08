//! Settings specific to the native platform

use std::{future::Future, pin::Pin};

use crate::app::platform;

use super::cloudflare::{add_address, CloudflareError};

use egui::vec2;
use egui_material_icons::icons;
use multiaddr::Multiaddr;

/// Cloudflare settings
/// CF_API_TOKEN, CF_ZONE_ID, CF_DOMAIN, CF_TXT_NAME   
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct CloudflareSettings {
    /// Cloudflare API Token.
    pub cf_api_token: String,
    /// Cloudflare Zone ID, ie. 123abc456d9ca31a8372d0c353
    pub cf_zone_id: String,
    /// Cloudflare Domain, ie. example.com
    pub cf_domain: String,
    /// Cloudflare TXT Name, ie. _dnsaddr.example.com
    pub cf_txt_name: String,

    /// Auto Update on startup
    pub auto_update: bool,
}

impl Default for CloudflareSettings {
    fn default() -> Self {
        Self {
            cf_api_token: "".to_string(),
            cf_zone_id: "".to_string(),
            cf_domain: "example.com".to_string(),
            cf_txt_name: "_dnsaddr.example.com".to_string(),
            auto_update: false,
        }
    }
}

/// Settings specific to the native platform
#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    /// Optional Cloudflare Settings
    pub cloudflare: CloudflareSettings,

    /// Whether the settings window is open
    open: bool,

    /// Updated on startup flag. Used to auto-update on startup.
    /// Skipped in serialization so the flag is not saved.
    #[serde(skip)]
    updated_on_startup: bool,
}

impl Settings {
    /// Show the current settings
    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, addr: &Multiaddr) {
        // auto update
        self.auto_update(addr);

        egui::Window::new("Cloudflare Options")
            .open(&mut self.open)
            .resizable(true)
            .show(ctx, |ui| {
                egui::Grid::new("my_grid")
                    .num_columns(2)
                    .min_col_width(100.0)
                    .max_col_width(600.0)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.add(egui::Hyperlink::from_label_and_url(
                            "Cloudflare API Token",
                            "https://dash.cloudflare.com/profile/api-tokens",
                        ));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.cloudflare.cf_api_token)
                                .desired_width(f32::INFINITY)
                                .password(true),
                        );
                        ui.end_row();

                        ui.add(egui::Hyperlink::from_label_and_url(
                            "Zone ID",
                            "https://dash.cloudflare.com/",
                        ));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.cloudflare.cf_zone_id)
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();

                        ui.add(egui::Hyperlink::from_label_and_url(
                            "Domain",
                            "https://dash.cloudflare.com/",
                        ));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.cloudflare.cf_domain)
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();

                        ui.add(egui::Hyperlink::from_label_and_url(
                            "TXT Name",
                            "https://dash.cloudflare.com/",
                        ));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.cloudflare.cf_txt_name)
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();

                        ui.label("Auto Update");
                        ui.checkbox(&mut self.cloudflare.auto_update, " on Startup");
                        ui.end_row();

                        // call add_address in a spawned block then display the log text once
                        // done.

                        ui.label("Multiaddr");
                        ui.vertical(|ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.monospace(addr.to_string());
                            });
                        });
                        ui.end_row();

                        ui.label("Manually Update");

                        ui.vertical(|ui| {
                            spawner(ctx, ui, || -> ClosureFut {
                                Box::pin(
                                    add_address()
                                        .api_token(self.cloudflare.cf_api_token.clone())
                                        .zone_id(self.cloudflare.cf_zone_id.clone())
                                        .txt_name(self.cloudflare.cf_txt_name.clone())
                                        .multiaddr(addr.clone())
                                        .call(),
                                )
                            });
                        });
                        ui.end_row();

                        ui.label("");
                        ui.label("");
                        ui.end_row();
                    });
            });

        if ui
            .add_sized(
                vec2(40.0, 40.0),
                egui::Button::new(
                    egui::RichText::new(icons::ICON_SETTINGS)
                        .size(16.0)
                        .family(egui::FontFamily::Proportional),
                )
                .min_size(vec2(2.0, 1.0)),
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            // Handle click event here
            // show a Window with the settings
            self.open = true;
        }
    }

    /// Auto-updates if auto_update is add_enabled
    /// spawns the future right away
    pub fn auto_update(&mut self, addr: &Multiaddr) {
        if self.cloudflare.auto_update && !self.updated_on_startup {
            self.updated_on_startup = true;
            let fut = Box::pin(
                add_address()
                    .api_token(self.cloudflare.cf_api_token.clone())
                    .zone_id(self.cloudflare.cf_zone_id.clone())
                    .txt_name(self.cloudflare.cf_txt_name.clone())
                    .multiaddr(addr.clone())
                    .call(),
            );

            platform::spawn(async move {
                // Fetch data
                tracing::trace!("Auto Updating");
                match fut.await {
                    Ok(data) => {
                        tracing::debug!("Auto updated data: {:?}", data);
                    }
                    Err(e) => {
                        tracing::error!("Auto update Error: {:?}", e);
                    }
                }
            });
        }
    }
}

//type FutStringList =
//    Pin<Box<dyn Future<Output = Result<Vec<String>, Box<dyn std::error::Error>>> + Send>>;

#[derive(Default, Clone)]
struct FetchState {
    response: Vec<String>,
    is_loading: bool,
    error: Option<String>,
}

type ClosureFut = Pin<Box<dyn Future<Output = Result<Vec<String>, CloudflareError>> + Send>>;

/// Takes care of resolving async operations and displaying the results
pub fn spawner(ctx: &egui::Context, ui: &mut egui::Ui, fut_closure: impl FnOnce() -> ClosureFut) {
    // Generate an id for the state
    let state_id = ui.id().with("future_fetch_state");

    // Retrieve shared fetch state
    let mut fetch_state =
        ctx.data_mut(|data| data.get_temp::<FetchState>(state_id).unwrap_or_default());

    if ui
        .add_enabled(!fetch_state.is_loading, egui::Button::new("Update Now"))
        .clicked()
    {
        tracing::debug!("Spawning async task");

        // Update fetch state
        fetch_state.response = Default::default();
        fetch_state.error = Default::default();
        fetch_state.is_loading = true;

        // Clone URL for async operation
        //let url = url.clone();
        let ctx_clone = ctx.clone();

        let mut fetch_state_clone = fetch_state.clone();

        let fut = fut_closure();

        platform::spawn(async move {
            // Fetch data
            match fut.await {
                Ok(data) => {
                    tracing::debug!("Data: {:?}", data);
                    fetch_state_clone.response = data;
                }
                Err(e) => {
                    tracing::error!("Error: {:?}", e);
                    fetch_state_clone.error = Some(format!("Error: {:?}", e));
                }
            }

            fetch_state_clone.is_loading = false;
            ctx_clone.data_mut(|data| {
                data.insert_temp(state_id, fetch_state_clone);
            });
            ctx_clone.request_repaint();
        });
    }

    // Loading indicator
    if fetch_state.is_loading {
        ui.spinner();
    }

    // Error display
    if let Some(error) = &fetch_state.error {
        ui.colored_label(egui::Color32::RED, error);
    }

    // Response display if lines
    if !fetch_state.response.is_empty() {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for line in &fetch_state.response {
                ui.label(line);
            }
        });
    }
}
