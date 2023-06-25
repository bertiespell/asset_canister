use auth::moderation::{BlockedStore, BLOCKED_STORE};
use candid::Deserialize;
use database::chunks::{ChunkID, CURRENT_CHUNK_ID};
use database::file::{FileID, CURRENT_FILE_ID};
use database::users::{UserStore, USER_STORE};
use ic_cdk::export::candid::CandidType;

use ic_cdk::storage;
use ic_cdk_macros::*;
use std::mem;

mod api;
mod auth;
mod controllers;
mod database;
mod metrics;
mod models;

#[init]
fn init() {
    ic_cdk::setup();
    CURRENT_FILE_ID.with(|current_id| *current_id.borrow_mut() = 0);
    CURRENT_CHUNK_ID.with(|current_id| *current_id.borrow_mut() = 0);
}

#[derive(Debug, CandidType, Deserialize)]
pub struct PreStableState {
    pub users: UserStore,
    pub current_file_id: FileID,
    pub current_chunk_id: ChunkID,
    // moderation
    pub blocked: BlockedStore,
}

#[derive(Debug, CandidType, Deserialize)]
pub struct PostStableState {
    pub users: UserStore,
    pub current_file_id: FileID,
    pub current_chunk_id: ChunkID,
    // moderation
    pub blocked: BlockedStore,
}

#[pre_upgrade]
fn pre_upgrade() {
    let users = USER_STORE.with(|state| mem::take(&mut *state.borrow_mut()));

    let current_file_id = CURRENT_FILE_ID.with(|state| mem::take(&mut *state.borrow_mut()));
    let current_chunk_id = CURRENT_CHUNK_ID.with(|state| mem::take(&mut *state.borrow_mut()));
    // moderation
    let blocked = BLOCKED_STORE.with(|state| mem::take(&mut *state.borrow_mut()));

    let stable_state = PreStableState {
        users,
        current_file_id,
        current_chunk_id,
        // moderation
        blocked,
    };

    storage::stable_save((stable_state,)).expect("Saving to stable store must succeed.");
}

#[post_upgrade]
fn post_upgrade() {
    let (PostStableState {
        users,
        current_file_id,
        current_chunk_id,
        // moderation
        blocked,
    },) = storage::stable_restore().expect("Failed to read network from stable memory.");

    USER_STORE.with(|state0| *state0.borrow_mut() = users);
    CURRENT_FILE_ID.with(|state0| *state0.borrow_mut() = current_file_id);
    CURRENT_CHUNK_ID.with(|state0| *state0.borrow_mut() = current_chunk_id);
    // moderation
    BLOCKED_STORE.with(|state0| *state0.borrow_mut() = blocked);
}
