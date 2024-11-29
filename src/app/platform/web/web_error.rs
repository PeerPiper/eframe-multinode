//! Web Errors

/// Error type for the web platform
#[derive(thiserror::Error, Debug)]
pub enum WebError {
    /// Error creatign OPFS Blockstore
    #[error("OPFS Blockstore Error: {0}")]
    OPFSBlockstore(String),

    /// Futures channel oneshot canceled
    #[error("Oneshot canceled")]
    OneshotCanceled(#[from] futures::channel::oneshot::Canceled),

    /// futures::futures_channel::mpsc::SendError
    #[error("SendError: {0}")]
    SendError(#[from] futures::channel::mpsc::SendError),

    /// from peerpiper_core::error::Error
    #[error("PeerPiper Core Error: {0}")]
    PeerPiperCoreError(#[from] peerpiper_core::error::Error),
}
