use std::{fmt::Display, ops::Deref};

use autosurgeon::{Hydrate, HydrateError, Prop, ReadDoc, Reconciler};
use bestsign_core::Base;
use multicid::EncodedVlad;

/// Provides the hydrate impl for Vlad
pub(super) fn hydrate<D: ReadDoc>(
    doc: &D,
    obj: &automerge::ObjId,
    prop: Prop<'_>,
) -> Result<VladId, HydrateError> {
    // Vlads can be turned into Vlad bytes.
    // This can be done byt `let vlad_bytes: Vec<u8> = vlad.into();`
    let inner = String::hydrate(doc, obj, prop)?;
    #[allow(clippy::unnecessary_fallible_conversions)]
    VladId::try_from(inner).map_err(|e| {
        HydrateError::unexpected("valid Vlad bytes", format!("Failed to parse Vlad: {}", e))
    })
}

pub(super) fn reconcile<R: Reconciler>(
    vlad_id: &VladId,
    mut reconciler: R,
) -> Result<(), R::Error> {
    reconciler.str(vlad_id)
}

/// A [VladId], which impls [AsRef<str>], since Vlad does not have a string representation,
/// and [autosurgeon::Reconcile] requires it for [std::collections::HashMap] keys.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct VladId {
    /// the Base58BTC encoded Vlad
    base58btc: String,
}

// Display
impl Display for VladId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.base58btc)
    }
}

impl VladId {
    /// Create a new VladId from a Base58BTC encoded string
    pub fn new(vlad: multicid::Vlad) -> Self {
        let base58btc = EncodedVlad::new(Base::Base58Btc, vlad).to_string();
        Self { base58btc }
    }
}

// let encoded = EncodedVlad::new(Base::Base36Lower, plog.vlad.clone()).to_string();
impl AsRef<str> for VladId {
    fn as_ref(&self) -> &str {
        &self.base58btc
    }
}

impl Deref for VladId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.base58btc
    }
}

impl From<multicid::Vlad> for VladId {
    fn from(vlad: multicid::Vlad) -> Self {
        Self::new(vlad)
    }
}

impl From<VladId> for multicid::Vlad {
    fn from(vlad_id: VladId) -> Self {
        EncodedVlad::try_from(vlad_id.as_ref())
            .expect("VladId should be a valid Base58BTC encoded Vlad")
            .to_inner()
    }
}

impl From<String> for VladId {
    /// Internal use only. Do not use as it's unchecked.
    /// Needed for [autosurgeon::Hydrate], even though String might not be a valid valid from outside
    fn from(s: String) -> Self {
        Self { base58btc: s }
    }
}

// impl TryFrom String, which asserts the base58btc encoding is valid first before creating a VladId
impl TryFrom<&str> for VladId {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        // check if the string is can decode into a vlad, then create a VladId from thos ebytes
        let vlad = EncodedVlad::try_from(s).map_err(|e| format!("Failed to parse Vlad: {}", e))?;
        Ok(Self::new(vlad.to_inner()))
    }
}
