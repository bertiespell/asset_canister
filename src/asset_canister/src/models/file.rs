use candid::Principal;
use ic_cdk::export::candid::{CandidType, Deserialize};
use serde_bytes::ByteBuf;
use sha3::{Digest, Sha3_256};
use std::collections::HashSet;

use crate::{
    api::file::FEFile,
    database::{chunks::ChunkID, file::FileID},
};

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq, Copy)]
pub enum FileType {
    PNG,
    JPEG,
    GIF,
    MP4,
    MOV,
    WEBP,
}

impl FileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileType::PNG => "image/png",
            FileType::JPEG => "image/jpeg",
            FileType::GIF => "image/gif",
            FileType::MP4 => "video/mp4",
            FileType::MOV => "video/quicktime",
            FileType::WEBP => "image/webp",
        }
    }

    pub fn convert_to_file_type(file_type: &str) -> Result<FileType, String> {
        match file_type {
            "image/png" => Ok(FileType::PNG),
            "image/jpeg" => Ok(FileType::JPEG),
            "image/gif" => Ok(FileType::GIF),
            "video/mp4" => Ok(FileType::MP4),
            "video/quicktime" => Ok(FileType::MOV),
            "image/webp" => Ok(FileType::WEBP),
            _ => Err(String::from("Unsupported file type")),
        }
    }

    pub fn url_slug(&self) -> &'static str {
        match self {
            FileType::PNG => "image",
            FileType::JPEG => "image",
            FileType::GIF => "image",
            FileType::MP4 => "video",
            FileType::MOV => "video",
            FileType::WEBP => "image",
        }
    }
}

pub type Hash = [u8; 32];

pub fn hash_bytes(value: impl AsRef<[u8]>) -> Hash {
    let mut hasher = Sha3_256::new();
    hasher.update(value.as_ref());
    hasher.finalize().into()
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct File {
    pub id: FileID,
    pub url: String,
    pub chunk_ids: Vec<ChunkID>,
    pub number_of_chunks: u64,
    pub file_name: String,
    pub file_type: FileType,
    pub owner: Principal,
    pub metadata: String,
    pub deleted_at: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
    pub accessors: HashSet<Principal>,
    pub hash: Hash,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct FileChunk {
    pub id: ChunkID,
    pub file_id: FileID,
    pub order_id: u64,
    pub chunk_data: ByteBuf,
    pub metadata: String,
    pub deleted_at: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
    pub hash: Hash,
    // todo: add version here
}

impl File {
    pub fn create_fe_type(&self) -> FEFile {
        FEFile {
            id: self.id,
            chunk_ids: self.chunk_ids.clone(),
            number_of_chunks: self.number_of_chunks,
            file_name: self.file_name.clone(),
            file_type: String::from(self.file_type.as_str()),
            owner: self.owner,
            metadata: String::from(self.metadata.as_str()),
            url: String::from(self.url.as_str()),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
