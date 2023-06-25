use candid::Principal;
use ic_cdk_macros::*;
use serde_bytes::ByteBuf;

use crate::{
    api::file::FEFile,
    auth::{
        file::{chunk_size_okay, file_size_accepted},
        user::get_logged_in_superuser,
    },
    database::file::create_file as be_create_file,
};

#[update]
pub fn prune_file(
    first_chunk: ByteBuf,
    file_name: String,
    number_of_chunks: u64,
    file_type: String,
    principal: Principal,
) -> Result<FEFile, String> {
    match get_logged_in_superuser() {
        Ok(_) => match file_size_accepted(number_of_chunks) {
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
