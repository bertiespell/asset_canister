use ic_cdk_macros::*;

use crate::{
    api::canister::CanisterInfo,
    auth::{
        canister::canister_storage_ok as be_canister_storage_ok, user::get_logged_in_superuser,
    },
    metrics::metrics::collect_metrics as be_collect_metrics,
};

#[update]
pub fn collect_metrics() -> Result<CanisterInfo, String> {
    match get_logged_in_superuser() {
        Ok(_) => Ok(be_collect_metrics()),
        Err(e) => Err(e),
    }
}

#[query]
pub fn canister_storage_ok() -> Result<u64, String> {
    match get_logged_in_superuser() {
        Ok(_) => be_canister_storage_ok(),
        Err(e) => Err(e),
    }
}
