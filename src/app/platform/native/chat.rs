//! Chat widget for the native platform using Ollama
//! By default uses Ollama::default() to connect to the Ollama server.

use egui::TextStyle;
use futures::StreamExt as _;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::Mutex as AsyncMutex;

use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage, ChatMessageResponseStream},
    Ollama,
};

use crate::app::platform;

/// Default model is llama3.3:latest
const DEFAULT_MODEL: &str = "llama3.3:latest";

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ChatWidget {
    /// Out Ollama instance.
    #[serde(skip)] // This how you opt-out of serialization of a field
    ollama: Arc<AsyncMutex<Ollama>>,
    /// Our prompt
    prompt: String,
    /// The current response
    response: Arc<RwLock<String>>,
    /// Previous response history
    history: Arc<Mutex<Vec<ChatMessage>>>,
}

impl Default for ChatWidget {
    fn default() -> Self {
        let ollama = Arc::new(AsyncMutex::new(Ollama::default()));

        let ollama_clone = ollama.clone();
        platform::spawn(async move {
            tracing::info!("Asserting Ollama model: {}", DEFAULT_MODEL);

            // check if ollama has this model, if not, pull it
            let ollama_lock = ollama_clone.lock().await;
            if let Ok(models) = ollama_lock.list_local_models().await {
                // check if Vec<LocalModel>'s field name matches the model we want'
                if !models.iter().any(|model| model.name == DEFAULT_MODEL) {
                    drop(ollama_lock);
                    let ollama_clone = ollama_clone.clone();
                    platform::spawn(async move {
                        if let Err(e) = ollama_clone
                            .lock()
                            .await
                            .pull_model(DEFAULT_MODEL.to_string(), false)
                            .await
                        {
                            tracing::error!("Failed to pull Ollama model: {:?}", e);
                        }
                    });
                } else {
                    tracing::info!("Ollama Model {} is already present", DEFAULT_MODEL);
                }
            }
        });
        Self {
            ollama,
            prompt: Default::default(),
            response: Default::default(),
            history: Default::default(),
        }
    }
}

impl ChatWidget {
    pub fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // Chat should look like:
        // At the verry bottom, the single line textedit with the Send button on the far right.
        // Filling the space above that, the chat history, with the most recent message at the bottom,
        // and the history scrolling up as new messages are added.
        // The chat history should be a scrollable region, with the most recent message at the bottom.
        // Since chat responses are streamed, when Send button is clicked we should start a new stream
        // by spawning a new tokio task to handle the stream.
        // When the stream response is complete, we should update the chat history with the response.
        // Scrollable chat history

        egui::TopBottomPanel::bottom("chat_prompt").show_inside(ui, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                // Reserve space for the button
                let button_width = ui.spacing().interact_size.x + ui.spacing().item_spacing.x;

                // Adjust the width of TextEdit to take up the remaining space
                ui.add_sized(
                    [ui.available_width() - button_width, 0.0],
                    egui::TextEdit::singleline(&mut self.prompt)
                        .text_color(egui::Color32::BLACK)
                        .font(TextStyle::Monospace)
                        .background_color(egui::Color32::from_rgb(0, 255, 0))
                        .hint_text("Ask me anything"),
                );

                // Add the button to the right of TextEdit
                if ui.button("Send").clicked() {
                    let ollama = self.ollama.clone();
                    let history = self.history.clone();
                    let response = self.response.clone();
                    let prompt = self.prompt.clone();
                    let ctx = ctx.clone();
                    tracing::info!("Sending chat message: {:?}", prompt);
                    tokio::spawn(async move {
                        start_chat(ctx, ollama, history, response, prompt).await;
                    });
                }
            });
            ui.add_space(8.0);
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical(|ui| {
                // History with a "Clear Hostory" button
                ui.horizontal(|ui| {
                    ui.label("Chat history:");
                    if ui.button("Clear History").clicked() {
                        self.history.lock().unwrap().clear();
                        self.response.write().unwrap().clear();
                    }
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for message in self.history.lock().unwrap().iter() {
                        ui.label(message.content.as_str());
                    }

                    // show the responses are they stream in
                    ui.label(self.response.read().unwrap().clone());
                });
            });
        });
    }
}

async fn start_chat(
    ctx: egui::Context,
    ollama: Arc<AsyncMutex<Ollama>>,
    history: Arc<Mutex<Vec<ChatMessage>>>,
    response: Arc<RwLock<String>>,
    input: String,
) {
    let mut stream: ChatMessageResponseStream = match ollama
        .lock()
        .await
        .send_chat_messages_with_history_stream(
            history.clone(),
            ChatMessageRequest::new(
                "llama3.1:latest".to_string(),
                vec![ChatMessage::user(input.to_string())],
            ),
        )
        .await
    {
        Ok(stream) => stream,
        Err(e) => {
            history
                .lock()
                .unwrap()
                .push(ChatMessage::user(format!("Error: {}", e)));
            return;
        }
    };

    while let Some(Ok(res)) = stream.next().await {
        tracing::info!("Got response: {:?}", res);
        let mut response_lock = response.write().unwrap();
        *response_lock += res.message.content.as_str();
        ctx.request_repaint();
    }

    // clear the response
    let mut response_lock = response.write().unwrap();
    *response_lock = "".to_string();
}
