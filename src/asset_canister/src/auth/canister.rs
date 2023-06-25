use crate::metrics::metrics::get_stable_memory_size;

use super::file::MAX_FILE_SIZE;

/// Max canister size is 16GB
/// Stable storage is being upgraded to 32GB but I'm not sure when this is
const MAX_SIZE: u64 = 4000000000 * 4;

/// Create a safety buffer of 2MB for heap computation
/// Not sure whether this is necessary
const SAFETY_BUFFER: u64 = 2000000;

/// Checks the canister has space to store a file of the maximum size
pub fn canister_storage_ok() -> Result<u64, String> {
    match get_stable_memory_size() + MAX_FILE_SIZE - SAFETY_BUFFER < MAX_SIZE {
        true => Ok(get_stable_memory_size()),
        false => Err(String::from("Canister is full")),
    }
}
