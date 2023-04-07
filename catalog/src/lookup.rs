use std::{io::{ Seek, BufReader, Write }, fmt::Display};
use binrw::{BinRead, BinWrite, BinReaderExt, BinResult, until_eof };

#[derive(BinRead, BinWrite, Default)]
#[brw(little)]
pub struct KeyData {
    pub count: u32,
    #[br(count = count)]
    pub entries: Vec<KeyDataValue>,
}

#[derive(BinRead, Debug)]
pub enum KeyDataValue {
    #[br(magic = 0u8)]
    String {
        length: u32,
        #[br(count = length, map = |x: Vec<u8>| String::from_utf8(x).unwrap())]
        string: String
    },
    #[br(magic = 4u8)]
    Hash(i32),
}

impl Display for KeyDataValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyDataValue::String { string, .. } => write!(f, "{}", string),
            KeyDataValue::Hash(hash) => write!(f, "{}", hash),
        }
    }
}

impl KeyDataValue {
    pub fn from_string<S: Into<String>>(internal_id: S) -> Self {
        let string = internal_id.into();
        KeyDataValue::String { length: string.len() as _, string }
    }

    pub fn get_size(&self) -> u32 {
        match self {
            KeyDataValue::String { length, .. } => *length + 5,
            KeyDataValue::Hash(_) => 5,
        }
    }
}

impl BinWrite for KeyDataValue {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        match self {
            KeyDataValue::Hash(hash) => {
                (4u8, hash).write_options(writer, endian, args)
            },
            KeyDataValue::String { string, .. } => {
                (0u8, string.len() as u32, string.as_bytes()).write_options(writer, endian, args)
            },
        }
    }
}

#[derive(BinRead, BinWrite, Default)]
#[brw(little)]
pub struct BucketData {
    pub count: u32,
    #[br(count = count)]
    pub entries: Vec<BucketEntry>,
}

#[derive(BinRead, BinWrite, Default, Debug)]
pub struct BucketEntry {
    pub key_data_offset: u32,
    pub count: u32,
    #[br(count = count)]
    pub indices: Vec<EntryId>,
}

#[derive(BinRead, BinWrite, Default)]
#[brw(little)]
pub struct EntryData {
    pub count: u32,
    #[br(count = count)]
    pub entries: Vec<EntryValue>,
}

#[derive(BinRead, BinWrite, Debug)]
pub struct EntryValue {
    pub internal_id: InternalId,
    pub provider_index: u32,
    pub dependency_key_idx: KeyId,
    pub dependency_hash: i32,
    pub data_index: ExtraId,
    pub primary_key: KeyId,
    pub resource_type: i32,
}

#[derive(BinRead, BinWrite, Default)]
#[brw(little)]
pub struct ExtraData {
    #[br(parse_with = until_eof)]
    // #[br(count = 2)]
    pub entries: Vec<ExtraValue>,
}

#[derive(BinRead, Default, Clone, Debug)]
#[brw(little)]
pub struct ExtraValue {
    // AsciiString,
    // UnicodeString,
    // UInt16,
    // UInt32,
    // Int32,
    // Hash128,
    // Type,
    // > JsonObject
    key_type: u8,
    assembly_name_len: u8,
    #[br(count = assembly_name_len, map = |x: Vec<u8>| String::from_utf8(x).unwrap())]
    assembly_name: String,
    class_name_len: u8,
    #[br(count = class_name_len, map = |x: Vec<u8>| String::from_utf8(x).unwrap())]
    class_name: String,
    json_len: i32,
    #[br(count = json_len, map = |x: Vec<u8>| String::from_utf8(x).unwrap())]
    json_text: String,
}

impl ExtraValue {
    pub fn get_size(&self) -> u32 {
        (1 + 1 + self.assembly_name.len() + 1 + self.class_name.len() + 4 + self.json_text.len()) as u32
    }
}

impl BinWrite for ExtraValue {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        //panic!("{:?}", encoding_rs::UTF_16LE.encode(&self.json_text).0.into_owned());
            (7u8, self.assembly_name.len() as u8, self.assembly_name.as_bytes(), self.class_name.len() as u8, self.class_name.as_bytes(), self.json_text.len() as i32, &self.json_text.as_bytes()).write_options(writer, endian, args)
    }
}

#[repr(transparent)]
#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InternalId(pub u32);

impl From<InternalId> for usize {
    fn from(index: InternalId) -> Self {
        index.0 as usize
    }
}

impl From<u32> for InternalId {
    fn from(index: u32) -> Self {
        InternalId(index)
    }
}

impl From<usize> for InternalId {
    fn from(index: usize) -> Self {
        InternalId(index as u32)
    }
}

#[repr(transparent)]
#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyId(pub i32);

impl From<KeyId> for isize {
    fn from(index: KeyId) -> Self {
        index.0 as isize
    }
}

impl From<i32> for KeyId {
    fn from(index: i32) -> Self {
        KeyId(index)
    }
}

impl From<isize> for KeyId {
    fn from(index: isize) -> Self {
        KeyId(index as i32)
    }
}

#[repr(transparent)]
#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryId(pub u32);

impl From<EntryId> for usize {
    fn from(index: EntryId) -> Self {
        index.0 as usize
    }
}

impl From<u32> for EntryId {
    fn from(index: u32) -> Self {
        EntryId(index)
    }
}

impl From<usize> for EntryId {
    fn from(index: usize) -> Self {
        EntryId(index as u32)
    }
}

#[repr(transparent)]
#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtraId(pub i32);

impl From<ExtraId> for isize {
    fn from(index: ExtraId) -> Self {
        index.0 as isize
    }
}

impl From<i32> for ExtraId {
    fn from(index: i32) -> Self {
        ExtraId(index)
    }
}

impl From<isize> for ExtraId {
    fn from(index: isize) -> Self {
        ExtraId(index as i32)
    }
}