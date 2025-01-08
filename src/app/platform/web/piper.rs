//! The PeerPiper Command sender for the Browser
#![allow(dead_code)]

use super::web_error::WebError as Error;
use peerpiper_browser::opfs::OPFSBlockstore;
use peerpiper_browser::Blockstore;
use send_wrapper::SendWrapper;
use std::ops::Deref;

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

impl Blockstore for OPFSWrapped {
    async fn get<const S: usize>(
        &self,
        cid: &cid::CidGeneric<S>,
    ) -> blockstore::Result<Option<Vec<u8>>> {
        tracing::debug!("Getting block from OPFS for CID: {:?}", cid);
        self.inner.deref().get(cid).await
    }

    async fn put_keyed<const S: usize>(
        &self,
        cid: &cid::CidGeneric<S>,
        data: &[u8],
    ) -> blockstore::Result<()> {
        self.inner.deref().put_keyed(cid, data).await
    }

    async fn remove<const S: usize>(&self, _cid: &cid::CidGeneric<S>) -> blockstore::Result<()> {
        //todo!();
        Ok(())
    }

    async fn close(self) -> blockstore::Result<()> {
        Ok(())
    }
}
