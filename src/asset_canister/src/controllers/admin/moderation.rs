use ic_cdk::export::Principal;
use ic_cdk_macros::*;

use crate::auth::moderation::{
    block_and_delete_user as be_block_and_delete_user, block_user as be_block_user,
    get_blocked_users as be_get_blocked_users, unblock_user as be_unblock_user,
};
use crate::auth::ratelimit::{get_warnings as be_get_warnings, Warning};
use crate::auth::user::get_logged_in_superuser;

#[update]
pub fn block_user(principal: Principal) -> Result<String, String> {
    match get_logged_in_superuser() {
        Ok(_) => {
            be_block_user(principal);
            Ok(String::from("User blocked"))
        }
        Err(e) => Err(e),
    }
}

#[update]
pub fn block_and_delete_user(principal: Principal) -> Result<String, String> {
    match get_logged_in_superuser() {
        Ok(_) => {
            be_block_and_delete_user(principal);
            Ok(String::from("User blocked and deleted"))
        }
        Err(e) => Err(e),
    }
}

#[update]
pub fn unblock_user(principal: Principal) -> Result<String, String> {
    match get_logged_in_superuser() {
        Ok(_) => {
            be_unblock_user(principal);
            Ok(String::from("User unblocked"))
        }
        Err(e) => Err(e),
    }
}

#[query]
pub fn get_warnings() -> Result<Vec<Warning>, String> {
    match get_logged_in_superuser() {
        Ok(_) => Ok(be_get_warnings()),
        Err(e) => Err(e),
    }
}

#[query]
pub fn get_blocked_users() -> Result<Vec<Principal>, String> {
    match get_logged_in_superuser() {
        Ok(_) => Ok(be_get_blocked_users()),
        Err(e) => Err(e),
    }
}
