use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::models::file::File;

use super::file::FileID;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct UserInfo {
    pub blocked: bool,
    pub files_owned: HashSet<FileID>,
    pub byte_limit: u64,
    pub bytes_used: u64,
}

pub type UserStore = HashMap<Principal, UserInfo>;

thread_local! {
    pub static USER_STORE: RefCell<UserStore> = RefCell::default();
}

pub fn get_user_info(principal: Principal) -> Result<UserInfo, String> {
    match USER_STORE.with(|user_store| user_store.borrow().get(&principal).cloned()) {
        Some(user_info) => Ok(user_info),
        None => Err(String::from("User not found")),
    }
}

pub fn update_user_info_file(
    principal: Principal,
    new_file: &File,
    bytes_used: u64,
) -> Result<Principal, String> {
    match get_user_info(principal) {
        Ok(user_info) => match USER_STORE.with(|user_store| {
            let mut files_owned = user_info.files_owned.clone();
            files_owned.insert(new_file.id);
            let new_bytes_used = user_info.bytes_used + bytes_used;

            user_store.borrow_mut().insert(
                principal,
                UserInfo {
                    files_owned,
                    blocked: user_info.blocked,
                    byte_limit: 0,
                    bytes_used: new_bytes_used,
                },
            )
        }) {
            Some(_) => Ok(principal),
            None => Ok(principal),
        },
        Err(_) => match USER_STORE.with(|user_store| {
            let mut files_owned = HashSet::new();
            files_owned.insert(new_file.id);
            user_store.borrow_mut().insert(
                principal,
                UserInfo {
                    files_owned,
                    blocked: false,
                    byte_limit: 0,
                    bytes_used,
                },
            )
        }) {
            Some(_) => Ok(principal),
            None => Ok(principal),
        },
    }
}

pub fn update_user_info_chunk(principal: Principal, bytes_used: u64) -> Result<Principal, String> {
    match get_user_info(principal) {
        Ok(user_info) => {
            match USER_STORE.with(|user_store| {
                let new_bytes_used = user_info.bytes_used + bytes_used;

                user_store.borrow_mut().insert(
                    principal,
                    UserInfo {
                        files_owned: user_info.files_owned,
                        blocked: user_info.blocked,
                        byte_limit: 0,
                        bytes_used: new_bytes_used,
                    },
                )
            }) {
                Some(_) => Ok(principal),
                None => Ok(principal),
            }
        }

        Err(e) => Err(e),
    }
}
