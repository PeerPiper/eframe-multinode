//! Native errors

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Futures channel oneshot canceled
    #[error("Oneshot canceled")]
    OneshotCanceled(#[from] futures::channel::oneshot::Canceled),

    /// futures::futures_channel::mpsc::SendError
    #[error("SendError: {0}")]
    SendError(#[from] futures::channel::mpsc::SendError),

    /// from peerpiper_core::error::Error
    #[error("PeerPiper Core Error: {0}")]
    PeerPiperCoreError(#[from] peerpiper::core::error::Error),
}
