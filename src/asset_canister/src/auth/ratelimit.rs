use ic_cdk::api::time;
use std::{cell::RefCell, collections::HashMap};

use candid::{CandidType, Deserialize, Principal};

use super::{moderation::block_user, user::get_logged_in_superuser};

pub type RateLimit = HashMap<Principal, Vec<Call>>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Call {
    time: u64,
    call_type: RateLimitMessageType,
    principal: Principal,
}

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq)]
pub struct Warning {
    number: i128,
    principal: Principal,
}

pub type Warnings = HashMap<Principal, Warning>;

thread_local! {
    pub static RATE_LIMIT_STORE: RefCell<RateLimit> = RefCell::default();
    pub static WARNED_USERS: RefCell<Warnings> = RefCell::default();
}

/// One day in nano seconds
pub const FILES_REFRESH_RATE_ONE_DAY: u64 = 86400000000000;
// How many we allow per day
pub const FILES_PER_DAY: u64 = 3;
// 5 minutes
pub const RATE_LIMIT: u64 = 300000000000;
// How many calls you're allowed to make in that window
pub const CALL_RATE_LIMIT_WINDOW: u64 = 50;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RateLimitMessageType {
    CreateFile,
    DeleteFile,
    UpdateFile,
    GetFile,
}

// TODO: add admin way to clear the rate_limit cache or do this in a heartbeat script

pub fn rate_limit(
    principal: Principal,
    message_type: RateLimitMessageType,
) -> Result<String, String> {
    let new_call = Call {
        time: time(),
        call_type: message_type,
        principal,
    };
    match get_logged_in_superuser() {
        Ok(_) => Ok(String::from("Superuser")),
        Err(_) => {
            let calls = get_calls(principal);
            match check_rate_limit(calls.clone()) {
                Ok(res) => {
                    if message_type == RateLimitMessageType::DeleteFile {
                        remove_last_create_call(principal, calls.clone());
                        insert_call(principal, new_call);
                        Ok(res)
                    } else if message_type == RateLimitMessageType::CreateFile {
                        match check_last_three_signals(calls.clone()) {
                            Ok(message) => {
                                insert_call(principal, new_call);
                                Ok(message)
                            }
                            Err(e) => Err(e),
                        }
                    } else {
                        insert_call(principal, new_call);
                        Ok(res)
                    }
                }
                Err(e) => {
                    if get_number_of_warnings(principal) > 200 {
                        insert_call(principal, new_call);
                        block_user(principal);
                        Err(e)
                    } else {
                        insert_call(principal, new_call);
                        insert_warning(principal);
                        Err(e)
                    }
                }
            }
        }
    }
}

pub fn get_warnings() -> Vec<Warning> {
    let mut warnings: Vec<Warning> = vec![];

    WARNED_USERS.with(|warning_store| {
        warning_store.borrow().iter().for_each(|(_key, warning)| {
            warnings.push(warning.clone());
        })
    });

    warnings
}

fn get_calls(principal: Principal) -> Vec<Call> {
    match RATE_LIMIT_STORE.with(|calls_store| calls_store.borrow().get(&principal).cloned()) {
        Some(calls) => calls,
        None => vec![],
    }
}

fn get_number_of_warnings(principal: Principal) -> i128 {
    match WARNED_USERS.with(|warning_store| warning_store.borrow().get(&principal).cloned()) {
        Some(warning) => warning.number,
        None => 0,
    }
}

fn insert_call(principal: Principal, call: Call) {
    let mut new_calls = get_calls(principal);
    new_calls.push(call);

    RATE_LIMIT_STORE.with(|rate_limit| rate_limit.borrow_mut().insert(principal, new_calls));
}

fn remove_last_create_call(principal: Principal, calls: Vec<Call>) {
    let mut new_calls = calls.clone();
    new_calls.sort_by(|a, b| b.time.cmp(&a.time));
    let index = new_calls
        .iter()
        .position(|call| call.call_type == RateLimitMessageType::CreateFile);

    match index {
        Some(index) => {
            new_calls.remove(index);

            RATE_LIMIT_STORE
                .with(|rate_limit| rate_limit.borrow_mut().insert(principal, new_calls));
        }
        // Ignore this - we could get into a state where we call delete but there is nothing in the rate limit store
        // Because the call has been cached
        None => (),
    }
}

fn insert_warning(principal: Principal) {
    let number_of_warnings = get_number_of_warnings(principal);

    WARNED_USERS.with(|warned_users| {
        warned_users.borrow_mut().insert(
            principal,
            Warning {
                number: number_of_warnings + 1,
                principal,
            },
        )
    });
}

// TODO: We should pull this whole file out and use the library instead,
// But it's worth noting there is a slight difference here in that it checks something slightly different
// I think we can put this in the library though to reuse code and have it all in just one place
fn check_last_three_signals(calls: Vec<Call>) -> Result<String, String> {
    let signal_calls = calls
        .iter()
        .filter(|call| call.call_type == RateLimitMessageType::CreateFile)
        .collect::<Vec<_>>();
    let mut sorted_calls = signal_calls.clone();
    sorted_calls.sort_by(|a, b| b.time.cmp(&a.time));

    match sorted_calls.get(FILES_PER_DAY as usize - 1) {
        Some(call) => {
            if less_than_a_day(call.time) {
                return Err(String::from("Daily image limit reached"));
            } else {
                Ok(String::from("Rate limit okay"))
            }
        }
        None => Ok(String::from("Rate limit okay")),
    }
}

fn check_rate_limit(calls: Vec<Call>) -> Result<String, String> {
    let mut sorted_calls = calls.clone();
    sorted_calls.sort_by(|a, b| b.time.cmp(&a.time));

    match sorted_calls.get(CALL_RATE_LIMIT_WINDOW as usize) {
        Some(call) => {
            if less_than_a_minute(call.time) {
                return Err(String::from("Rate limit reached"));
            } else {
                Ok(String::from("Rate limit okay"))
            }
        }
        None => Ok(String::from("Rate limit okay")),
    }
}

pub fn less_than_a_minute(call_time: u64) -> bool {
    (time() - call_time) < RATE_LIMIT
}

pub fn less_than_a_day(call_time: u64) -> bool {
    (time() - call_time) < FILES_REFRESH_RATE_ONE_DAY
}
