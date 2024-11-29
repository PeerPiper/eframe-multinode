//! Web Widget(s) for the web platform.
use crate::app::platform;
use eframe::wasm_bindgen::prelude::*;
use eframe::web_sys::{Request, RequestInit, RequestMode, Response};
use wasm_bindgen_futures::JsFuture;

#[derive(Debug, serde::Deserialize)]
struct DnsTXTQuery {
    /// 'answer' is 'Answer' in the DNS TXT response json
    #[serde(rename = "Answer")]
    answer: Vec<Answer>,
}

#[derive(Debug, serde::Deserialize)]
struct Answer {
    data: String,
}

pub async fn fetch_dns_query(domain: String) -> Result<Vec<String>, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    // take /dnsaddr/peerpiper.io and convert to _dnsaddr.peerpiper.io
    let domain = domain.replace("/dnsaddr/", "_dnsaddr.");
    let url = format!("https://1.1.1.1/dns-query?type=16&name={domain}");

    let request = Request::new_with_str_and_init(&url, &opts)?;
    request.headers().set("Accept", "application/dns-json")?;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    let json = JsFuture::from(resp.json()?).await?;
    let multiaddrs: DnsTXTQuery = serde_wasm_bindgen::from_value(json)?;

    log::debug!("multiaddrs: {:?}", multiaddrs);

    let multiaddrs: Vec<String> = multiaddrs
        .answer
        .iter()
        .map(|a| a.data.replace("dnsaddr=", "").trim_matches('"').to_string())
        .collect();
    Ok(multiaddrs)
}

#[derive(Default, Clone)]
struct FetchState {
    response: Vec<String>,
    is_loading: bool,
    error: Option<String>,
}

pub fn fetch(ctx: &egui::Context, ui: &mut egui::Ui, url: &mut String) {
    // Generate an id for the state
    let state_id = ui.id().with("fetch_state");

    // Retrieve shared fetch state
    let mut fetch_state =
        ctx.data_mut(|data| data.get_temp::<FetchState>(state_id).unwrap_or_default());

    ui.label("URL:");
    ui.add_sized([300.0, 20.0], egui::TextEdit::singleline(url));

    if ui.button("Fetch").clicked() {
        // Update fetch state
        fetch_state.response = Default::default();
        fetch_state.error = Default::default();
        fetch_state.is_loading = true;

        // Clone URL for async operation
        let url = url.clone();
        let ctx_clone = ctx.clone();

        let mut fetch_state_clone = fetch_state.clone();

        platform::spawn(async move {
            // Fetch data
            match fetch_dns_query(url).await {
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
}
