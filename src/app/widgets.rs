//! Custom Widgets for this app
use std::{future::Future, pin::Pin};

use super::platform::spawn;

// A wrapper that allows the more idiomatic usage pattern: `ui.add(…)`
/// HTTP asset Fetcher and result displayer.
///
/// ## Example:
/// ``` ignore
/// ui.add(fetcher(ctx, async {
///    let resp = reqwest::get("https://httpbin.org/ip").await?;
///    let body = resp.text().await?;
///    Ok(vec![body])
///    }));
/// ```
pub fn fetcher(ctx: &egui::Context, fut: FutStringList) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| fetch(ctx, ui, fut)
}

#[derive(Default, Clone)]
struct FetchState {
    response: Vec<String>,
    is_loading: bool,
    error: Option<String>,
}

type FutStringList =
    Pin<Box<dyn Future<Output = Result<Vec<String>, Box<dyn std::error::Error>>> + Send>>;

/// Takes a Function with signature  async fn( Params ) -> Result<Vec<String>, Error> and Params
pub fn fetch(ctx: &egui::Context, ui: &mut egui::Ui, fut: FutStringList) -> egui::Response {
    // Generate an id for the state
    let state_id = ui.id().with("fetch_state");

    // Retrieve shared fetch state
    let mut fetch_state =
        ctx.data_mut(|data| data.get_temp::<FetchState>(state_id).unwrap_or_default());

    let result = ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui.button("Fetch").clicked() {
            // Update fetch state
            fetch_state.response = Default::default();
            fetch_state.error = Default::default();
            fetch_state.is_loading = true;

            // Clone for async operation
            let ctx_clone = ctx.clone();

            let mut fetch_state_clone = fetch_state.clone();

            spawn(async move {
                // Fetch data
                match fut.await {
                    Ok(data) => {
                        fetch_state_clone.response = data;
                    }
                    Err(e) => {
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

        // Response display
        egui::ScrollArea::vertical().show(ui, |ui| {
            for line in &fetch_state.response {
                ui.label(line);
            }
        });
    });

    result.response
}
