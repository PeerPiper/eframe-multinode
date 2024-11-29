use serde::{Deserialize, Serialize};
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct SerializableArcMutexVec<T>(Arc<Mutex<Vec<T>>>);

impl Default for SerializableArcMutexVec<String> {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(Vec::new())))
    }
}

impl Deref for SerializableArcMutexVec<String> {
    type Target = Arc<Mutex<Vec<String>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Serialize + for<'de> Deserialize<'de>> Serialize for SerializableArcMutexVec<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.lock().unwrap().serialize(serializer)
    }
}

impl<'de, T: Serialize + for<'d> Deserialize<'d>> Deserialize<'de> for SerializableArcMutexVec<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vec = Vec::deserialize(deserializer)?;
        Ok(SerializableArcMutexVec(Arc::new(Mutex::new(vec))))
    }
}
