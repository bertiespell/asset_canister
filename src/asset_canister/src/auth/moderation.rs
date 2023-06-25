use candid::CandidType;
use ic_cdk::export::Principal;
use serde::{Deserialize as SerdeDe, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

use crate::database::file::delete_all_files_by_prinicipal;

// TODO: move this into a library to avoid code duplication between packages

#[derive(Clone, Debug, CandidType, PartialEq, Serialize, SerdeDe)]
pub struct Blocked {
    pub principal: Principal,
    pub metadata: String,
}

pub type BlockedStore = HashMap<Principal, Blocked>;

thread_local! {
    pub static BLOCKED_STORE: RefCell<BlockedStore> = RefCell::default();
}

pub fn is_blocked(principal: Principal) -> bool {
    match BLOCKED_STORE.with(|blocked_store| blocked_store.borrow().get(&principal).cloned()) {
        Some(_) => true,
        None => false,
    }
}

pub fn get_blocked_users() -> Vec<Principal> {
    let mut blocked_users: Vec<Principal> = vec![];

    BLOCKED_STORE.with(|blocked_store| {
        blocked_store
            .borrow()
            .iter()
            .for_each(|(principal, _warning)| {
                blocked_users.push(*principal);
            })
    });

    blocked_users
}

pub fn block_and_delete_user(principal: Principal) {
    block_user(principal);
    delete_all_files_by_prinicipal(principal);
}

pub fn block_user(principal: Principal) {
    BLOCKED_STORE.with(|block_store| {
        block_store.borrow_mut().insert(
            principal,
            Blocked {
                principal,
                metadata: String::from(""),
            },
        )
    });
}

pub fn unblock_user(principal: Principal) {
    BLOCKED_STORE.with(|block_store| block_store.borrow_mut().remove(&principal));
}
