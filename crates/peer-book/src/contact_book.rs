#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
mod autosurgeon_vlad;

use autosurgeon::{Hydrate, Reconcile};
pub(crate) use autosurgeon_vlad::VladId;
use std::collections::HashMap;

// Define the Contact struct with #[key] attribute on the id field
#[derive(Debug, Clone, Reconcile, Hydrate, PartialEq, bon::Builder)]
pub(crate) struct Contact {
    #[key] // Mark the id field as the unique key
    #[autosurgeon(with = "autosurgeon_vlad")]
    // because Vlad is a foreign type, we need to specify the reconcile path
    /// The unique identifier for the contact
    id: VladId,
    /// The nickname/petname of the contact
    name: String,
    /// The some notes about contact
    notes: String,
}

impl Contact {
    /// The contact id
    pub(crate) fn id(&self) -> VladId {
        self.id.clone()
    }

    /// The contact name
    pub(crate) fn name(&self) -> String {
        self.name.clone()
    }

    /// The contact notes
    pub(crate) fn notes(&self) -> String {
        self.notes.clone()
    }
}

// Contact into Vec<String> for passing to Rhai
impl From<Contact> for Vec<String> {
    fn from(contact: Contact) -> Vec<String> {
        vec![contact.id.to_string(), contact.name, contact.notes]
    }
}

// Define the ContactBook struct
#[derive(Default, Debug, Clone, Reconcile, Hydrate, PartialEq)]
pub(crate) struct ContactBook {
    contacts: HashMap<VladId, Contact>,
}

impl ContactBook {
    /// Add a new contact to the contact book
    pub(crate) fn add(&mut self, contact: Contact) {
        self.contacts.insert(contact.id.clone(), contact);
    }

    /// Update an existing contact
    pub(crate) fn update(&mut self, contact: Contact) {
        self.contacts.insert(contact.id.clone(), contact);
    }

    /// Remove a contact by id
    pub(crate) fn remove(&mut self, id: VladId) {
        self.contacts.remove(&id);
    }

    /// Returns a list of all contacts
    pub(crate) fn contacts(&self) -> Vec<Contact> {
        self.contacts.values().cloned().collect()
    }
}

impl From<ContactBook> for Vec<Vec<String>> {
    fn from(book: ContactBook) -> Vec<Vec<String>> {
        book.contacts
            .values()
            .map(|contact| {
                vec![
                    contact.id.to_string(),
                    contact.name.clone(),
                    contact.notes.clone(),
                ]
            })
            .collect()
    }
}

// Book from Vec<Vec<String>>
impl From<Vec<Contact>> for ContactBook {
    fn from(contacts: Vec<Contact>) -> Self {
        let contacts = contacts
            .into_iter()
            .map(|contact| (contact.id.clone(), contact))
            .collect();
        ContactBook { contacts }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use automerge::AutoCommit;
    use autosurgeon::{hydrate, reconcile};
    use bestsign_core::Codec;
    use multihash::mh;
    use multikey::nonce;

    #[test]
    fn test_add_contact() {
        // build a nonce
        let mut rng = rand::rngs::OsRng;
        let nonce = nonce::Builder::new_from_random_bytes(32, &mut rng)
            .try_build()
            .unwrap();

        // build a cid
        let cid = multicid::cid::Builder::new(Codec::Cidv1)
            .with_target_codec(Codec::DagCbor)
            .with_hash(
                &mh::Builder::new_from_bytes(Codec::Sha2256, b"for great justice, move every zig!")
                    .unwrap()
                    .try_build()
                    .unwrap(),
            )
            .try_build()
            .unwrap();

        let vlad = multicid::vlad::Builder::default()
            .with_nonce(&nonce)
            .with_cid(&cid)
            .try_build(|cid| {
                // sign those bytes
                let v: Vec<u8> = cid.clone().into();
                Ok(v)
            })
            .unwrap();

        let mut contact_book = ContactBook {
            contacts: HashMap::new(),
        };
        let contact = Contact {
            id: VladId::new(vlad),
            name: "John Doe".to_string(),
            notes: "Some notes about John Doe".to_string(),
        };

        contact_book.add(contact.clone());

        assert_eq!(contact_book.contacts.len(), 1);
        assert_eq!(contact_book.contacts.get(&contact.id), Some(&contact));

        // Create a new Automerge document
        let mut doc = AutoCommit::new();

        // Apply changes to the Automerge document
        // takes the Rust value and updates an automerge document to match the value
        reconcile(&mut doc, &contact_book).unwrap();

        let saved = doc.save();
        let doc2 = AutoCommit::load(&saved).unwrap();

        // Hydrate an instance of ContactBook from the Automerge document
        // creates a rust value given an automerge document
        let copy_of_book: ContactBook = hydrate(&doc2).unwrap();

        assert_eq!(copy_of_book, contact_book);

        // confirm this works with the lock of LazyLock<Mutex<ContactBook>>
        let arc_book = std::sync::Arc::new(std::sync::Mutex::new(contact_book.clone()));
        let lock = arc_book.lock().unwrap();
        let mut doc3 = AutoCommit::new();
        reconcile(&mut doc3, &*lock).unwrap();

        let saved_lock = doc3.save();

        let doc4 = AutoCommit::load(&saved_lock).unwrap();

        let copy_of_book2: ContactBook = hydrate(&doc4).unwrap();

        assert_eq!(copy_of_book2, contact_book);

        // check the document
    }
}
