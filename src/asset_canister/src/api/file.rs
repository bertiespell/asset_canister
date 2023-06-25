use candid::Principal;
use ic_cdk::export::candid::{CandidType, Deserialize};

use crate::database::{chunks::ChunkID, file::FileID};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct FEFile {
    pub id: FileID,
    pub chunk_ids: Vec<ChunkID>,
    pub number_of_chunks: u64,
    pub file_name: String,
    pub file_type: String,
    pub owner: Principal,
    pub metadata: String,
    pub url: String,
    pub created_at: u64,
    pub updated_at: u64,
}
