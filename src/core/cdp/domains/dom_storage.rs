use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Item = Vec<String>;
pub type SerializedStorageKey = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageId {
    #[serde(rename = "securityOrigin")]
    pub origin: Option<String>,
    #[serde(rename = "storageKey")]
    pub storage_key: Option<SerializedStorageKey>,
    #[serde(rename = "isLocalStorage")]
    pub is_local_storage: bool,
}

//Requests
#[derive(Serialize)]
pub struct DomStorageEnable {}

impl DomStorageEnable {
    pub fn default() -> Self {
        Self {}
    }
}

#[derive(Serialize)]
pub struct DomStorageDisable {}

impl DomStorageDisable {
    pub fn default() -> Self {
        Self {}
    }
}

#[derive(Serialize)]
pub struct RemoveDOMStorageItem<'a> {
    #[serde(rename = "storageId")]
    storage_id: StorageId,
    #[serde(rename = "key")]
    key: &'a str,
}

impl<'a> RemoveDOMStorageItem<'a> {
    pub fn default(storage_id: &StorageId, key: &'a str) -> Self {
        Self {
            storage_id: storage_id.clone(),
            key,
        }
    }
}

#[derive(Serialize)]
pub struct GetDOMStorageItems {
    #[serde(rename = "storageId")]
    storage_id: StorageId,
}

impl GetDOMStorageItems {
    pub fn default(storage_id: &StorageId) -> Self {
        Self {
            storage_id: storage_id.clone(),
        }
    }
}

#[derive(Deserialize)]
pub struct GetDOMStorageItemsResponse {
    #[serde(rename = "entries")]
    pub entries: Vec<Item>,
}

//Events
#[derive(Deserialize, Debug, Clone)]
pub struct DomStorageItemAdded {
    #[serde(rename = "storageId")]
    pub storage_id: StorageId,
    #[serde(rename = "key")]
    pub key: Item,
    #[serde(rename = "newValue")]
    pub value: Item,
}
