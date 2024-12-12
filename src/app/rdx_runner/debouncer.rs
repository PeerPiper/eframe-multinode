use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub(crate) struct Debouncer {
    pub(crate) cancel_token: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    pub(crate) callback: Arc<dyn Fn() + Send + Sync>,
    pub(crate) delay: Duration,
}

impl Debouncer {
    #[allow(dead_code)]
    pub(crate) fn new<F>(callback: F, delay_ms: u64) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        Debouncer {
            cancel_token: Arc::new(Mutex::new(None)),
            callback: Arc::new(callback),
            delay: Duration::from_millis(delay_ms),
        }
    }

    pub(crate) async fn debounce(&self) {
        // Cancel any existing timer
        {
            let mut current_token = self.cancel_token.lock().await;
            if let Some(sender) = current_token.take() {
                let _ = sender.send(()); // Signal to cancel previous timer
            }
        }

        // Create a new cancellation channel
        let (sender, receiver) = tokio::sync::oneshot::channel();

        // Store the new cancel sender
        {
            let mut current_token = self.cancel_token.lock().await;
            *current_token = Some(sender);
        }

        // Clone what we need for the async block
        let callback = self.callback.clone();
        let delay = self.delay;

        // Spawn a task that can be cancelled
        tokio::spawn(async move {
            tokio::select! {
                _ = sleep(delay) => {
                    callback();
                }
                _ = receiver => {
                    // Cancelled, do nothing
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_debouncer() {
        // create a debouncer with a delay of 100delay_ms
        // and a callback that increments a counter
        // after a final 400ms delay, count should be one

        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        // avoid async closures

        let callback = move || {
            let counter = counter_clone.clone();
            tokio::spawn(async move {
                let mut counter = counter.lock().await;
                *counter += 1;
            });
        };

        let debouncer = Debouncer::new(callback, 100);

        debouncer.debounce().await;
        sleep(Duration::from_millis(50)).await;
        debouncer.debounce().await;
        sleep(Duration::from_millis(50)).await;
        debouncer.debounce().await;
        sleep(Duration::from_millis(50)).await;
        debouncer.debounce().await;
        sleep(Duration::from_millis(400)).await;

        let counter = counter.lock().await;
        assert_eq!(*counter, 1);
    }
}
