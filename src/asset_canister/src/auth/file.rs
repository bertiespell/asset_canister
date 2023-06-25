// Add rate limiting, block list and max image size

use candid::Principal;
use ic_cdk::caller;

use crate::{
    database::{
        chunks::get_all_chunks_for_file as be_get_all_chunks_for_file,
        file::{get_file_by_id, FileID},
    },
    models::file::File,
};

use super::{
    canister::canister_storage_ok,
    moderation::is_blocked,
    ratelimit::{rate_limit, RateLimitMessageType},
    user::{get_logged_in_principal, get_logged_in_superuser},
};

/// Max file size is currently 11.4MB (total of 6 chunks * 1.9MB chunk size)
pub const MAX_FILE_SIZE: u64 = 11400000;

/// We allow a max file size of 11.4MB, which is 6 chunks
pub const MAX_CHUNKS: u64 = 6;

/// Chunk size is set just below the 2MB message limit at 1.9MB
pub const CHUNK_SIZE: u64 = 1900000;

/// Checks the proposed number of chunks is under the allowed amount
pub fn file_size_accepted(number_of_chunks: u64) -> Result<Principal, String> {
    match get_logged_in_superuser() {
        // let admins save larger files
        Ok(principal) => Ok(principal),
        Err(_) => {
            if number_of_chunks <= MAX_CHUNKS {
                return Ok(caller());
            } else {
                return Err(String::from("Number of chunks exceeds file size"));
            }
        }
    }
}

/// Checks the caller owns the file
pub fn caller_owns_file_or_is_superuser(file_id: FileID) -> Result<File, String> {
    match get_logged_in_principal() {
        Ok(_) => match get_file_by_id(&file_id) {
            // let admins access files
            Ok(file) => match get_logged_in_superuser() {
                Ok(_) => Ok(file),
                Err(_) => {
                    if file.owner == caller() {
                        return Ok(file);
                    } else {
                        return Err(String::from("Caller is not the owner of the file"));
                    }
                }
            },
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

/// Checks that the caller
/// Is authenticated
/// Is not rate limited
/// Is not blocked
/// Has not created too many files
/// That the canister has space to accept the file
pub fn caller_accepted(call_type: RateLimitMessageType) -> Result<Principal, String> {
    match get_logged_in_principal() {
        Ok(principal) => match rate_limit(principal, call_type) {
            Ok(_) => match is_blocked(principal) {
                false => match canister_storage_ok() {
                    Ok(_) => Ok(principal),
                    Err(e) => Err(e),
                },
                true => Err(String::from("Unauthorized")),
            },
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

/// Check the number of chunks *already* saved with this FileID isn't reached
pub fn chunks_within_file_size(file: &File) -> Result<String, String> {
    let number_of_chunks = file.number_of_chunks.clone();
    match be_get_all_chunks_for_file(&file) {
        Ok(chunks) => match chunks.len() <= number_of_chunks as usize {
            true => Ok(String::from("Chunks are within filesize")),
            false => Err(String::from("All chunks already allocated for file")),
        },
        Err(e) => Err(e),
    }
}

/// Check the number of bytes within a chunk is acceptable
/// This is actually enforced by canister message size limitation
/// But let's re-enforce here just to be sure
pub fn chunk_size_okay(number_of_bytes: usize) -> Result<Principal, String> {
    if number_of_bytes <= CHUNK_SIZE as usize {
        return Ok(caller());
    } else {
        return Err(String::from("Too many bytes in chunk"));
    }
}
