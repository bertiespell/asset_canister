use candid::Principal;

use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::btreemap::InsertError;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use serde_bytes::ByteBuf;
use std::collections::HashSet;
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;

use crate::models::file::{hash_bytes, File, FileType};

const MAX_KEY_SIZE: u32 = 8;
const MAX_VALUE_SIZE: u32 = 20000000;

use super::chunks::{insert_chunk, remove_chunk};
use super::users::update_user_info_file;

pub type FileID = u64;

// For a type to be used in a `StableBTreeMap`, it needs to implement the `Storable`
// trait, which specifies how the type can be serialized/deserialized.
//
// In this example, we're using candid to serialize/deserialize the struct, but you
// can use anything as long as you're maintaining backward-compatibility. The
// backward-compatibility allows you to change your struct over time (e.g. adding
// new fields).
//
// The `Storable` trait is already implemented for many common types (e.g. u64, String),
// so you can use those directly without implementing the `Storable` trait for them.
impl Storable for File {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Vec<u8>) -> Self {
        Decode!(&bytes, Self).unwrap()
    }
}

thread_local! {
    pub static CURRENT_FILE_ID: RefCell<FileID> = RefCell::default();
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static FILE_MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static FILE_MAP: RefCell<StableBTreeMap<Memory, FileID, File>> = RefCell::new(
        StableBTreeMap::init(
            FILE_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            MAX_KEY_SIZE,
            MAX_VALUE_SIZE
        )
    );
}

pub fn create_file(
    first_chunk: ByteBuf,
    file_name: String,
    number_of_chunks: u64,
    file_type: String,
    owner: Principal,
) -> Result<File, String> {
    let bytes_used = first_chunk.len() as u64;
    match FileType::convert_to_file_type(file_type.as_str()) {
        Ok(file_type) => CURRENT_FILE_ID.with(|current_id| {
            let id = *current_id.borrow_mut();
            *current_id.borrow_mut() = id + 1;
            let hash = hash_bytes(&first_chunk);

            match insert_chunk(id, first_chunk, 0) {
                Ok(chunk_id) => {
                    let url;
                    if option_env!("DFX_NETWORK") == Some("local") {
                        url = format!(
                            "http://{}.localhost:4943/{}/{}",
                            ic_cdk::api::id(),
                            file_type.url_slug(),
                            id
                        );
                    } else {
                        url = format!(
                            "https://{}.raw.ic0.app/{}/{}",
                            ic_cdk::api::id(),
                            file_type.url_slug(),
                            id
                        );
                    }

                    let created_at = time();

                    let mut accessors = HashSet::new();
                    accessors.insert(owner);

                    let file = File {
                        id,
                        url,
                        chunk_ids: vec![chunk_id],
                        number_of_chunks,
                        file_name,
                        file_type,
                        owner,
                        metadata: String::from(""),
                        deleted_at: None,
                        created_at,
                        updated_at: created_at,
                        accessors,
                        hash,
                    };

                    match insert_file(file.id, file.clone()) {
                        Ok(_) => match update_user_info_file(owner, &file, bytes_used) {
                            Ok(_) => Ok(file),
                            Err(e) => Err(e),
                        },
                        Err(e) => Err(String::from(e.to_string())),
                    }
                }
                Err(e) => Err(e),
            }
        }),
        Err(e) => Err(e),
    }
}

pub fn delete_file(file_id: FileID) -> Result<String, String> {
    match get_file_by_id(&file_id) {
        Ok(file) => {
            let chunks_to_delete = file.chunk_ids.clone();

            remove_file(&file_id);

            chunks_to_delete.into_iter().for_each(|chunk| {
                remove_chunk(chunk);
            });

            Ok(String::from("File deleted"))
        }
        Err(e) => Err(e),
    }
}

pub fn delete_all_files_by_prinicipal(principal: Principal) -> Vec<File> {
    let mut all_files: Vec<File> = vec![];

    // TODO: this needs fixing since it will not work
    // iterating over the file map exceeds the execution limit
    // So we need to store a way of mapping principals to files, so that we can retrieve the files by ID one by one
    // Rather than looping over the whole array
    FILE_MAP.with(|p| {
        p.borrow().iter().for_each(|(_, file)| {
            if file.owner == principal {
                all_files.push(file.clone())
            }
        });
    });

    all_files
}

pub fn get_current_file_id() -> u64 {
    CURRENT_FILE_ID.with(|current_id| *current_id.borrow_mut())
}

pub fn get_file_by_id(file_id: &FileID) -> Result<File, String> {
    match get_file(file_id) {
        Some(file) => Ok(file),
        None => Err(String::from("file not found")),
    }
}

pub fn get_file(key: &FileID) -> Option<File> {
    FILE_MAP.with(|p| p.borrow().get(&key))
}

pub fn insert_file(key: FileID, value: File) -> Result<Option<File>, InsertError> {
    FILE_MAP.with(|p| p.borrow_mut().insert(key, value))
}

pub fn remove_file(key: &FileID) -> Option<File> {
    // TODO: instead of removing this, let's set soft-delete to true instead
    FILE_MAP.with(|p| p.borrow_mut().remove(key))
}
