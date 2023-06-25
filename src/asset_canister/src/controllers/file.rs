use crate::api::file::FEFile;
use crate::auth::file::{
    caller_accepted, caller_owns_file_or_is_superuser, chunk_size_okay, chunks_within_file_size,
    file_size_accepted,
};
use crate::auth::ratelimit::{rate_limit, RateLimitMessageType};

use crate::database::chunks::{
    get_chunk_by_id as be_get_chunk_by_id, put_chunk as be_put_chunk, ChunkID,
};
use crate::database::file::{
    create_file as be_create_file, delete_file as be_delete_file,
    get_current_file_id as be_get_current_file_id, get_file_by_id as be_get_file_by_id, FileID,
};
use crate::models::file::FileChunk;
use ic_cdk_macros::*;
use serde_bytes::ByteBuf;

#[update]
pub fn create_file(
    first_chunk: ByteBuf,
    file_name: String,
    number_of_chunks: u64,
    file_type: String,
) -> Result<FEFile, String> {
    match caller_accepted(RateLimitMessageType::CreateFile) {
        Ok(principal) => match file_size_accepted(number_of_chunks) {
            Ok(_) => match chunk_size_okay(first_chunk.len() as usize) {
                Ok(_) => match be_create_file(
                    first_chunk,
                    file_name,
                    number_of_chunks,
                    file_type,
                    principal,
                ) {
                    Ok(file) => Ok(file.create_fe_type()),
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        },

        Err(e) => Err(e),
    }
}

#[update]
pub fn delete_file(file_id: FileID) -> Result<String, String> {
    match caller_accepted(RateLimitMessageType::DeleteFile) {
        Ok(principal) => match caller_owns_file_or_is_superuser(file_id) {
            Ok(_) => match rate_limit(principal, RateLimitMessageType::DeleteFile) {
                Ok(_) => be_delete_file(file_id),
                Err(e) => Err(e),
            },

            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

#[update]
pub fn put_chunk(file_id: FileID, chunk: ByteBuf, order_id: u64) -> Result<FEFile, String> {
    match caller_accepted(RateLimitMessageType::UpdateFile) {
        Ok(_) => match caller_owns_file_or_is_superuser(file_id) {
            Ok(file) => match chunks_within_file_size(&file) {
                Ok(_) => match chunk_size_okay(chunk.len()) {
                    Ok(_) => match be_put_chunk(&file, chunk, order_id) {
                        Ok(_) => Ok(file.create_fe_type()),
                        Err(e) => Err(e),
                    },
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

#[query]
pub fn get_chunk_by_id(chunk_id: ChunkID) -> Result<FileChunk, String> {
    be_get_chunk_by_id(chunk_id)
}

#[query]
pub fn get_current_file_id() -> Result<u64, String> {
    Ok(be_get_current_file_id())
}

#[query]
pub fn get_file_by_id(file_id: FileID) -> Result<FEFile, String> {
    match be_get_file_by_id(&file_id) {
        Ok(file) => Ok(file.create_fe_type()),
        Err(e) => Err(e),
    }
}
