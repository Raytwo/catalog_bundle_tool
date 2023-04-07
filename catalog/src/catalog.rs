use binrw::{BinResult, BinRead, BinWrite, meta::WriteEndian};
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use thiserror::Error;
use base64;

use rand::{self, Rng};

use crate::lookup::{InternalId, KeyData, BucketData, EntryData, ExtraData, KeyId, KeyDataValue, BucketEntry, EntryId, EntryValue, ExtraId, ExtraValue};

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("a filesystem error happened: {0}")]
    Io(#[from] std::io::Error),
    #[error("a json parsing error happened: {0}")]
    Json(#[from] serde_json::Error),
    #[error("a decoding error happened: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("a internalid with this string already exists")]
    DuplicateInternalId,
    #[error("a internalid with this string does not exist")]
    MissingInternalId,
}

fn serialize_catalog_table<T, S>(v: T, serializer: S) -> Result<S::Ok, S::Error>
where
T: BinWrite<Args<'static> = ()>,
S: Serializer {
    let mut buff = std::io::Cursor::new(Vec::new());
    v.write_le_args(&mut buff, ()).map_err(serde::ser::Error::custom)?;
    let base = base64::encode(buff.get_ref());

    serializer.serialize_str(&base)
}

fn deserialize_catalog_table<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
T: BinRead<Args<'static> = ()>,
D: Deserializer<'de> {
    let buf = String::deserialize(deserializer)?;
    let buf = base64::decode(&buf).map_err(CatalogError::Base64Decode).map_err(serde::de::Error::custom)?; 

    T::read_le_args(&mut std::io::Cursor::new(buf), ()).map_err(serde::de::Error::custom)
}


#[derive(Deserialize, Serialize)]
pub struct Catalog {
    m_LocatorId: String,
    m_InstanceProviderData: ProviderData,
    m_SceneProviderData: ProviderData,
    m_ResourceProviderData: Vec<ProviderData>,
    m_ProviderIds: Vec<String>,
    pub m_InternalIds: Vec<String>,
    #[serde(deserialize_with = "deserialize_catalog_table", serialize_with = "serialize_catalog_table")]
    pub m_KeyDataString: KeyData,
    #[serde(deserialize_with = "deserialize_catalog_table", serialize_with = "serialize_catalog_table")]
    m_BucketDataString: BucketData,
    #[serde(deserialize_with = "deserialize_catalog_table", serialize_with = "serialize_catalog_table")]
    m_EntryDataString: EntryData,
    #[serde(deserialize_with = "deserialize_catalog_table", serialize_with = "serialize_catalog_table")]
    m_ExtraDataString: ExtraData,
    m_resourceTypes: Vec<ObjectType>,
    m_InternalIdPrefixes: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct ProviderData {
    m_Id: String,
    m_ObjectType: ObjectType,
    m_Data: String,
}

#[derive(Deserialize, Serialize)]
pub struct ObjectType {
    m_AssemblyName: String,
    pub m_ClassName: String,
}

impl Catalog {
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, CatalogError> {
        let catalog_str = &std::fs::read_to_string(path.as_ref())?;
        serde_json::from_str(catalog_str).map_err(CatalogError::Json)
    }

    pub fn from_str<S: AsRef<str>>(string: S) -> Result<Self, CatalogError> {
        serde_json::from_str(string.as_ref()).map_err(CatalogError::Json)
    }

    pub fn from_slice<S: AsRef<[u8]>>(slice: S) -> Result<Self, CatalogError> {
        serde_json::from_slice(slice.as_ref()).map_err(CatalogError::Json)
    }

    pub fn get_internal_id_index<S: AsRef<str>>(&self, internal_id: S) -> Option<InternalId> {
        self.m_InternalIds
        .iter()
        .position(|x| x == internal_id.as_ref())
        .map(InternalId::from)
    }

    pub fn get_internal_id_from_index<I: Into<usize>>(&self, index: I) -> Option<&String> {
        self.m_InternalIds.get(index.into())
    }

    pub fn get_internal_ids(&self) -> Vec<String> {
        self.m_InternalIds.clone()
    }

    pub fn get_key(&self, id: KeyId) -> Option<&KeyDataValue> {
        self.m_KeyDataString.entries.get(isize::from(id) as usize)
    }

    pub fn get_bucket(&self, id: KeyId) -> Option<&BucketEntry> {
        self.m_BucketDataString.entries.get(isize::from(id) as usize)
    }

    pub fn get_bucket_mut(&mut self, id: KeyId) -> Option<&mut BucketEntry> {
        self.m_BucketDataString.entries.get_mut(isize::from(id) as usize)
    }

    pub fn get_entry(&self, id: EntryId) -> Option<&EntryValue> {
        self.m_EntryDataString.entries.get(usize::from(id) as usize)
    }

    pub fn get_entry_by_internal_id(&self, id: InternalId) -> Option<&EntryValue> {
        self.m_EntryDataString.entries.iter().find(|x| x.internal_id == id)
    }

    pub fn get_entry_id_by_internal_id(&self, id: InternalId) -> Option<usize> {
        self.m_EntryDataString.entries.iter().position(|x| x.internal_id == id)
    }

    pub fn get_extra(&self, id: ExtraId) -> Option<&ExtraValue> {
        self.m_ExtraDataString.entries.get(isize::from(id) as usize)
    }

    pub fn get_dependencies(&self, entry: &EntryValue) -> Option<&[EntryId]> {
        Some(&self.get_bucket(entry.dependency_key_idx)?.indices)
    }

    pub fn add_internalid<S: AsRef<str>>(&mut self, internal_id: S) -> Result<InternalId, CatalogError> {
        if self.get_internal_id_index(&internal_id).is_none() {
            self.m_InternalIds.push(String::from(internal_id.as_ref()));
            Ok((self.m_InternalIds.len() - 1).into())
        } else {
            Err(CatalogError::DuplicateInternalId)
        }
    }

    pub fn get_next_key_offset(&self) -> u32 {
        self.m_BucketDataString.entries.last().unwrap().key_data_offset + self.m_KeyDataString.entries.last().unwrap().get_size()
    }

    pub fn get_next_extra_offset(&self) -> u32 {
        self.m_ExtraDataString.entries.iter().map(|extra| extra.get_size()).sum()
    }

    pub fn get_unique_hash(&self) -> i32 {
        let mut rng = rand::thread_rng();
        let mut unique_value: i32 = rng.gen();

        while self.m_KeyDataString.entries.iter().filter_map(|entry| {
            match entry {
                KeyDataValue::String { .. } => None,
                KeyDataValue::Hash(hash) => Some(hash),
            }
        }).any(|entry| entry == &unique_value) {
            unique_value = rng.gen();
        }

        unique_value
    }

    pub fn add_key(&mut self, key: KeyDataValue) -> KeyId {
        let key_data_offset = self.get_next_key_offset();

        // Add the dependency
        self.m_KeyDataString.count += 1;
        self.m_KeyDataString.entries.push(key);

        // Get the current amount of EntryData entries, as our new entry will use the next index
        self.m_BucketDataString.count += 1;
        self.m_BucketDataString.entries.push(BucketEntry { key_data_offset, count: 1, indices: vec![EntryId(self.m_EntryDataString.entries.len() as u32)] });

        KeyId((self.m_KeyDataString.count - 1) as i32)
    }

    pub fn add_dependency_key(&mut self, key: KeyDataValue, dependencies: &[EntryId]) -> KeyId {
        let key_data_offset = self.get_next_key_offset();

        // Add the dependency
        self.m_KeyDataString.count += 1;
        self.m_KeyDataString.entries.push(key);

        self.m_BucketDataString.count += 1;
        self.m_BucketDataString.entries.push(BucketEntry { key_data_offset, count: dependencies.len() as u32, indices: dependencies.to_vec() });

        KeyId((self.m_KeyDataString.count - 1) as i32)
    }

    pub fn add_extra_data(&mut self, extra: ExtraValue) -> ExtraId {
        let offset = self.get_next_extra_offset();
        // Add new extra entry
        self.m_ExtraDataString.entries.push(extra);

        // TODO: Make a method to calculate the size of the table. add_extradata
        ExtraId(offset as i32)
    }

    pub fn add_bundle<S: AsRef<str>>(&mut self, internal_id: S, key: S, extra: ExtraValue) -> Result<(), CatalogError> {
        // Try to add the internal ID, return a Duplicate error if it already exists
        // TODO: This should be a method that combines both
        let iid = self.add_internalid(&internal_id)?;
        let primary_key = self.add_key(KeyDataValue::from_string(key.as_ref()));

        let new_entry = EntryValue { 
            internal_id: iid,
            provider_index: 0,
            dependency_key_idx: KeyId(-1),
            dependency_hash: 0,
            data_index: self.add_extra_data(extra),
            primary_key,
            resource_type: 0,
        };

        // Add new entry
        self.m_EntryDataString.count += 1;
        self.m_EntryDataString.entries.push(new_entry);

        Ok(())
    } 

    pub fn add_prefab<S: AsRef<str>>(&mut self, internal_id: S, key: S, dependencies: &[String]) -> Result<(), CatalogError> {
        // TODO: This should be a method that combines both
        // Try to add the internal ID, return a Duplicate error if it already exists
        let iid = self.add_internalid(&internal_id)?;
        let primary_key = self.add_key(KeyDataValue::from_string(key.as_ref()));

        let hash = self.get_unique_hash();

        // Dependency stuff
        // TODO: Turn this into a lookup method
        let indices: Vec<EntryId> = dependencies.iter().flat_map(|dep| self.get_internal_id_index(dep)).flat_map(|id| self.get_entry_id_by_internal_id(id)).map(EntryId::from).collect();
        // TODO: Generate the hash randomly. It cannot already exist in the Key table, so make sure it is unique.
        let dependency_key_idx = self.add_dependency_key(KeyDataValue::Hash(hash), &indices);

        let new_entry = EntryValue { 
            internal_id: iid,
            provider_index: 2,
            dependency_key_idx: dependency_key_idx,
            // TODO: Add a check to make sure the hash here matches with the dependency entry
            dependency_hash: hash,
            data_index: ExtraId(-1),
            primary_key,
            resource_type: 4,
        };

        // Add new entry
        self.m_EntryDataString.count += 1;
        self.m_EntryDataString.entries.push(new_entry);

        Ok(())
    } 
}