use ic_cdk::export::Principal;

include!("../../../../env/admins.rs");

pub fn get_logged_in_principal() -> Result<Principal, String> {
    let caller = ic_cdk::api::caller();
    // The anonymous principal is not allowed to do certain actions
    if caller == Principal::anonymous() {
        return Err(String::from(
            "Anonymous principal not allowed to make calls.",
        ));
    }

    Ok(caller)
}

pub fn get_logged_in_superuser() -> Result<Principal, String> {
    let caller = ic_cdk::api::caller();
    // The anonymous principal is not allowed to do certain actions
    if caller == Principal::anonymous() {
        return Err(String::from(
            "Anonymous principal not allowed to make calls.",
        ));
    }

    if option_env!("DFX_NETWORK") == Some("local") {
        match DEV_ADMINS.contains(&caller.to_string().as_str()) {
            true => Ok(caller),
            false => Err(String::from("Unauthorised")),
        }
    } else {
        match PROD_ADMINS.contains(&caller.to_string().as_str()) {
            true => Ok(caller),
            false => Err(String::from("Unauthorised")),
        }
    }
}
