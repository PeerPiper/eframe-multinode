//! Native bindings for PeerPiper commander
use crate::app::platform::peerpiper::PeerPiper;
use peerpiper_native::{NativeBlockstoreBuilder, NativeError};

/// Creates a new PeerPiper instance with [NativeBlockstore]
pub async fn create_peerpiper() -> Result<PeerPiper, NativeError> {
    let handler = NativeBlockstoreBuilder::default().open().await?;
    Ok(PeerPiper::new(handler))
}
