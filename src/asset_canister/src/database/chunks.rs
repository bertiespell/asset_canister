use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::btreemap::InsertError;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use serde_bytes::ByteBuf;
use std::vec;
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;

use crate::models::file::{hash_bytes, File, FileChunk};

const MAX_KEY_SIZE: u32 = 8;
// This value can potentially break
// We can store 1900000 bytes in the u8 data
// But we also need to leave some bytes as room for the other fields on a chunk
// e.g. metadata is a String field.
// TODO: we should have validation/checks on the chunk as it is written
// to ensure that any string fields (or the total of all dynamic fields) remains under the max value saved in memory here
const MAX_VALUE_SIZE: u32 = 2000000;

use super::file::{insert_file, FileID};
use super::users::update_user_info_chunk;

pub type ChunkID = u64;

impl Storable for FileChunk {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Vec<u8>) -> Self {
        // CBOR? serde-default
        Decode!(&bytes, Self).unwrap()
    }
}

thread_local! {
    pub static CURRENT_CHUNK_ID: RefCell<ChunkID> = RefCell::default();

    static CHUNK_MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static CHUNK_MAP: RefCell<StableBTreeMap<Memory, ChunkID, FileChunk>> = RefCell::new(
        StableBTreeMap::init(
            CHUNK_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            MAX_KEY_SIZE,
            MAX_VALUE_SIZE
        )
    );
}

fn get(key: ChunkID) -> Option<FileChunk> {
    CHUNK_MAP.with(|p| p.borrow().get(&key))
}

fn insert(key: ChunkID, value: FileChunk) -> Result<Option<FileChunk>, InsertError> {
    CHUNK_MAP.with(|p| p.borrow_mut().insert(key, value))
}

pub fn remove_chunk(key: ChunkID) -> Option<FileChunk> {
    // TODO: instead of removing this, let's set soft-delete to true instead
    CHUNK_MAP.with(|p| p.borrow_mut().remove(&key))
}

/// Inserts a chunk into the store and updates the file to include reference to a chunk
pub fn put_chunk<'a>(file: &'a File, chunk: ByteBuf, order_id: u64) -> Result<&'a File, String> {
    let bytes_used = chunk.len() as u64;
    match insert_chunk(file.id, chunk, order_id) {
        Ok(chunk_id) => {
            let mut chunk_ids = file.chunk_ids.clone();

            chunk_ids.push(chunk_id);
            let updated_file = File {
                id: file.id,
                url: (*file.url).to_string(),
                number_of_chunks: file.number_of_chunks,
                file_name: (*file.file_name).to_string(),
                chunk_ids,
                file_type: file.file_type,
                owner: file.owner,
                metadata: (*file.metadata).to_string(),
                deleted_at: file.deleted_at,
                created_at: file.created_at,
                updated_at: time(),
                accessors: file.accessors.clone(),
                hash: file.hash,
            };

            match insert_file(file.id, updated_file.clone()) {
                Ok(_) => match update_user_info_chunk(file.owner, bytes_used) {
                    Ok(_) => Ok(file),
                    Err(e) => Err(e),
                },
                Err(e) => Err(String::from(e.to_string())),
            }
        }
        Err(e) => Err(e),
    }
}

// Inserts a chunk into the store
pub fn insert_chunk(
    file_id: FileID,
    chunk_data: ByteBuf,
    order_id: u64,
) -> Result<ChunkID, String> {
    CURRENT_CHUNK_ID.with(|current_chunk_id| {
        let id = *current_chunk_id.borrow_mut();
        *current_chunk_id.borrow_mut() = id + 1;

        let created_at = time();

        let hash = hash_bytes(&chunk_data);

        let file_chunk = FileChunk {
            file_id,
            id,
            chunk_data,
            order_id,
            metadata: String::from(""),
            deleted_at: None,
            created_at,
            updated_at: created_at,
            hash,
        };

        match insert(file_chunk.id, file_chunk.clone()) {
            Ok(None) => Ok(file_chunk.id),
            Ok(Some(_)) => ic_cdk::trap("Attempting to overwrite chunk on insert"),
            Err(e) => Err(String::from(e.to_string())),
        }
    })
}

pub fn get_chunk_by_id(chunk_id: ChunkID) -> Result<FileChunk, String> {
    match get(chunk_id) {
        Some(chunk) => Ok(chunk),
        None => Err(String::from("file not found")),
    }
}

pub fn get_all_chunks_for_file(file: &File) -> Result<Vec<FileChunk>, String> {
    let mut all_chunks: Vec<FileChunk> = vec![];

    file.chunk_ids.iter().for_each(|chunk_id| {
        CHUNK_MAP.with(|p| {
            let found_chunk = p.borrow().get(&chunk_id);
            if found_chunk.is_some() {
                all_chunks.push(found_chunk.unwrap());
            }
        });
    });

    return Ok(all_chunks);
}

pub fn get_chunk_by_order_id_for_file(file: &File, order_id: u64) -> Option<FileChunk> {
    let mut found_chunk: Option<FileChunk> = None;

    let mut found_chunks: Vec<FileChunk> = vec![];
    file.chunk_ids.iter().for_each(|chunk_id| {
        CHUNK_MAP.with(|p| {
            let found_chunk = p.borrow().get(&chunk_id);
            if found_chunk.is_some() {
                found_chunks.push(found_chunk.unwrap());
            }
        });
    });

    found_chunks.iter().for_each(|chunk| {
        if chunk.order_id == order_id {
            found_chunk = Some(chunk.clone())
        }
    });

    return found_chunk;
}
