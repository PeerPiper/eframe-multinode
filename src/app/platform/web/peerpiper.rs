//! The PeerPiper Command sender for the Browser
#![allow(dead_code)]

use super::web_error::WebError as Error;
use crate::app::platform::peerpiper::PeerPiper;
use crate::app::Cid;
use peerpiper_browser::opfs::OPFSBlockstore;
use send_wrapper::SendWrapper;
use std::ops::Deref;

/// Creates a new PeerPiper instance with [NativeBlockstore]
pub async fn create_peerpiper() -> Result<PeerPiper, Error> {
    log::info!("Creating PeerPiper with OPFSBlockstore wrapped");
    let handler = OPFSWrapped::new().await?;
    Ok(PeerPiper::new(handler))
}

/// A Wrapper sturct around OPFSBlockstore so that we can make it Send
#[derive(Debug, Clone)]
pub struct OPFSWrapped {
    inner: SendWrapper<OPFSBlockstore>,
}

impl OPFSWrapped {
    pub async fn new() -> Result<Self, Error> {
        let handler = OPFSBlockstore::new()
            .await
            .map_err(|e| Error::OPFSBlockstore(e.as_string().unwrap_or_default()))?;
        Ok(Self {
            inner: SendWrapper::new(handler),
        })
    }
}

// impl SystemCommandHandler for OPFSWrapped:
impl peerpiper::core::SystemCommandHandler for OPFSWrapped {
    type Error = Error;

    async fn put(&self, bytes: Vec<u8>) -> Result<Cid, Error> {
        // use send_wrapper get_ref to execute the inner function
        Ok(self.inner.deref().put(bytes).await?)
    }

    async fn get(&self, key: Vec<u8>) -> Result<Vec<u8>, Error> {
        Ok(self.inner.deref().get(key).await?)
    }
}
