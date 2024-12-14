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

    /// from peerpiper::core::error::Error
    #[error("PeerPiper Core Error: {0}")]
    PeerPiperCoreError(#[from] peerpiper::core::error::Error),

    /// from peerpiper_browser::error::Error
    #[error("PeerPiper Browser Error: {0}")]
    PeerPiperBrowserError(#[from] peerpiper_browser::Error),
    ///// CommanderNotReady
    //#[error("Commander not ready. You might be trying to send commands before it is ready.")]
    //CommanderNotReady,
}
