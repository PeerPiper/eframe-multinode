//! Web Widget(s) for the web platform.

use eframe::wasm_bindgen::prelude::*;
use eframe::web_sys::{Request, RequestInit, RequestMode, Response};
use wasm_bindgen_futures::{spawn_local, JsFuture};

use egui_suspense::EguiSuspense;

/// The Connect Widget
pub fn connect(dnsaddr: &mut String) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| connect_ui(ui, dnsaddr)
}

/// Serde struct for: curl --header 'accept: application/dns-json' 'https://1.1.1.1/dns-query?type=16&name=cloudflare.com'
// {"Status":0,"TC":false,"RD":true,"RA":true,"AD":false,"CD":false,"Question":[{"name":"_dnsaddr.peerpiper.io","type":16}],"Answer":[{"name":"_dnsaddr.peerpiper.io","type":16,"TTL":300,"data":"\"dnsaddr=/ip6/2607:fea8:fec0:8526::2c81/udp/42069/webrtc-direct/certhash/uEiDEYac45bQ3DMYhAgNf5Bu-xYRDVkVgmmeJwfyEERZ4bw/p2p/12D3KooWSsrJqCDVunhDq3bV6LGSVE2f1i4xbE47483jJTPgbTED\""},{"name":"_dnsaddr.peerpiper.io","type":16,"TTL":300,"data":"\"dnsaddr=/ip6/2607:fea8:fec0:8526::2c81/udp/50562/webrtc-direct/certhash/uEiDEYac45bQ3DMYhAgNf5Bu-xYRDVkVgmmeJwfyEERZ4bw/p2p/12D3KooWSsrJqCDVunhDq3bV6LGSVE2f1i4xbE47483jJTPgbTED\""},{"name":"_dnsaddr.peerpiper.io","type":16,"TTL":300,"data":"\"dnsaddr=/ip6/2607:fea8:fec0:8526::2c81/udp/51642/webrtc-direct/certhash/uEiDEYac45bQ3DMYhAgNf5Bu-xYRDVkVgmmeJwfyEERZ4bw/p2p/12D3KooWSsrJqCDVunhDq3bV6LGSVE2f1i4xbE47483jJTPgbTED\""}]}
#[derive(serde::Deserialize)]
struct DnsTXTQuery {
    //status: u8,
    //tc: bool,
    //rd: bool,
    //ra: bool,
    //ad: bool,
    //cd: bool,
    //question: Vec<Question>,
    answer: Vec<Answer>,
}

//#[derive(serde::Deserialize)]
//struct Question {
//    name: String,
//    #[serde(rename = "type")]
//    qtype: u8,
//}

#[derive(serde::Deserialize)]
struct Answer {
    //name: String,
    //#[serde(rename = "type")]
    //atype: u8,
    //ttl: u32,
    data: String,
}

pub async fn fetch_dns_query(domain: String) -> Result<String, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    // take /dnsaddr/peerpiper.io and convert to _dnsaddr.peerpiper.io
    let domain = domain.replace("/dnsaddr/", "_dnsaddr.");
    let url = format!("https://1.1.1.1/dns-query?type=16&name={domain}");
    //let url = "https://1.1.1.1/dns-query?type=16&name=cloudflare.com";

    log::info!("Fetching DNS query for: {}", url);

    let request = Request::new_with_str_and_init(&url, &opts)?;
    request.headers().set("Accept", "application/dns-json")?;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    //let json = JsFuture::from(resp.json()?).await?;
    let text = JsFuture::from(resp.text()?).await?;
    //    let multiaddrs: DnsTXTQuery = serde_wasm_bindgen::from_value(json)?;
    // let multiaddrs: Vec<String> = multiaddrs.answer.iter().map(|a| a.data.clone()).collect();
    Ok(text
        .as_string()
        .ok_or_else(|| JsValue::from_str("no text"))?)
}

#[allow(clippy::ptr_arg)] // false positive
pub fn connect_ui(ui: &mut egui::Ui, dnsaddr: &mut String) -> egui::Response {
    // Generate an id for the state
    let state_id = ui.id().with("shown");

    // Get state for this widget.
    let mut shown = ui.data_mut(|d| d.get_temp::<String>(state_id).unwrap_or_default());

    log::info!("Shown: {:?}", shown);

    let result = ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if shown.is_empty() {
            let addr = dnsaddr.to_string();
            let mut suspense = EguiSuspense::reloadable(move |cb| {
                let addr = addr.clone();
                spawn_local(async move {
                    let multiaddrs = fetch_dns_query(addr).await.map_err(|e| {
                        log::error!("Error fetching DNS query: {:?}", e);
                    });
                    cb(if multiaddrs.is_ok() {
                        let data = multiaddrs.unwrap();
                        log::info!("Got DNS query: {:?}", data);
                        Ok(format!("Got DNS query: {:?}", data))
                    } else {
                        Err("No DNS found!".to_string())
                    });
                });
            });
            suspense.ui(ui, |ui, data, state| {
                // Update the persistent state here
                ui.data_mut(|d| d.insert_temp(state_id, data.clone()));

                log::info!("CB Data: {:?}", data);
                ui.monospace(format!("Data: {:?}", data));

                if ui.button("Reload").clicked() {
                    state.reload();
                }
            })
        } else {
            // show the data, multiline text
            ui.monospace(shown.as_str());
            Some(if ui.button("Clear").clicked() {
                shown.clear();
                // Update the persistent state here
                ui.data_mut(|d| d.insert_temp(state_id, shown.clone()));
            })
        }
    });

    log::info!("Result: {:?}", result);

    // All done! Return the interaction response so the user can check what happened
    // (hovered, clicked, …) and maybe show a tooltip:
    result.response
}

#[derive(Default, Clone)]
struct FetchState {
    response: String,
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
        fetch_state.response = String::new();
        fetch_state.error = None;
        fetch_state.is_loading = true;

        // Clone URL for async operation
        let url = url.clone();
        let ctx_clone = ctx.clone();

        let mut fetch_state_clone = fetch_state.clone();

        wasm_bindgen_futures::spawn_local(async move {
            // Fetch data
            match fetch_dns_query(url).await {
                Ok(data) => {
                    fetch_state_clone.response = data;
                }
                Err(e) => {
                    fetch_state_clone.error = Some(format!("Error: {:?}", e));
                    fetch_state_clone.response = format!("Error: {:?}", e);
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
        ui.add_sized(
            [ui.available_width(), 300.0],
            egui::TextEdit::multiline(&mut fetch_state.response)
                .desired_rows(10)
                .lock_focus(true),
        );
    });
}
