// Based in (large) part on https://github.com/tailscale/pam/blob/main/cmd/pam_tailscale/src/pam.rs (BSD-3-Clause Licensed)
#[cfg(feature = "chrono")]
use chrono::prelude::*;
use moon_phase;
use std::env;
#[cfg(not(feature = "chrono"))]
use std::time::SystemTime;

use moon_phase::MoonPhase;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_uint};

// Start from tailscale/pam
pub type PamHandleT = *const c_uint;
pub type PamFlags = c_uint;
pub type PamResult<T> = Result<T, PamResultCode>;

pub const PAM_SILENT: PamFlags = 0x8000;

#[allow(non_camel_case_types, dead_code)]
#[derive(Debug)]
#[repr(C)]
pub enum PamItemType {
    PAM_SERVICE = 1,
    PAM_USER = 2,
    PAM_TTY = 3,
    PAM_RHOST = 4,
    PAM_CONV = 5,
    PAM_AUTHTOK = 6,
    PAM_OLDAUTHTOK = 7,
    PAM_RUSER = 8,
    PAM_USER_PROMPT = 9,
    PAM_FAIL_DELAY = 10,
    PAM_XDISPLAY = 11,
    PAM_XAUTHDATA = 12,
    PAM_AUTHTOK_TYPE = 13,
}

#[allow(non_camel_case_types, dead_code)]
#[derive(Debug)]
#[repr(C)]
pub enum PamResultCode {
    PAM_SUCCESS = 0,
    PAM_OPEN_ERR = 1,
    PAM_SYMBOL_ERR = 2,
    PAM_SERVICE_ERR = 3,
    PAM_SYSTEM_ERR = 4,
    PAM_BUF_ERR = 5,
    PAM_PERM_DENIED = 6,
    PAM_AUTH_ERR = 7,
    PAM_CRED_INSUFFICIENT = 8,
    PAM_AUTHINFO_UNAVAIL = 9,
    PAM_USER_UNKNOWN = 10,
    PAM_MAXTRIES = 11,
    PAM_NEW_AUTHTOK_REQD = 12,
    PAM_ACCT_EXPIRED = 13,
    PAM_SESSION_ERR = 14,
    PAM_CRED_UNAVAIL = 15,
    PAM_CRED_EXPIRED = 16,
    PAM_CRED_ERR = 17,
    PAM_NO_MODULE_DATA = 18,
    PAM_CONV_ERR = 19,
    PAM_AUTHTOK_ERR = 20,
    PAM_AUTHTOK_RECOVERY_ERR = 21,
    PAM_AUTHTOK_LOCK_BUSY = 22,
    PAM_AUTHTOK_DISABLE_AGING = 23,
    PAM_TRY_AGAIN = 24,
    PAM_IGNORE = 25,
    PAM_ABORT = 26,
    PAM_AUTHTOK_EXPIRED = 27,
    PAM_MODULE_UNKNOWN = 28,
    PAM_BAD_ITEM = 29,
    PAM_CONV_AGAIN = 30,
    PAM_INCOMPLETE = 31,
}

// End from tailscale/pam

#[derive(Debug, Eq, PartialEq)]
pub enum PamFullmoonAction {
    Deny,
    Allow,
}

impl TryFrom<String> for PamFullmoonAction {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "deny" => Ok(Self::Deny),
            "allow" => Ok(Self::Allow),
            _ => Err(()),
        }
    }
}

#[cfg(feature = "chrono")]
fn get_moon_phase() -> MoonPhase {
    MoonPhase::new(Utc::now())
}

#[cfg(not(feature = "chrono"))]
fn get_moon_phase() -> MoonPhase {
    MoonPhase::new(SystemTime::now())
}

fn is_fullmoon() -> bool {
    get_moon_phase().phase_name == moon_phase::Phase::Full
}

// Start from tailscale/pam
fn extract_argv(argc: c_int, argv: *const *const c_char) -> Vec<String> {
    (0..argc)
        .map(|o| unsafe {
            CStr::from_ptr(*argv.offset(o as isize) as *const c_char)
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}
// End from tailscale/pam

fn extract_action(args: Vec<String>) -> PamFullmoonAction {
    args.into_iter()
        .find(|s| s.starts_with("action="))
        .unwrap_or(String::from("deny"))
        .replacen("action=", "", 1)
        .try_into()
        .unwrap_or(PamFullmoonAction::Deny)
}

fn debug_print<S: Into<String>>(msg: S, flags: PamFlags) {
    let env_set = env::var("WHY_IS_THIS_NOT_WORKING").is_ok();
    if env_set && (flags & PAM_SILENT) == 0 {
        println!("{}", msg.into());
    }
}

// Functions signatures and stuff are also from tailscale/pam
#[no_mangle]
pub extern "C" fn pam_sm_acct_mgmt(
    _: PamHandleT,
    flags: PamFlags,
    argc: c_int,
    argv: *const *const c_char,
) -> PamResultCode {
    let args = extract_argv(argc, argv);
    let action = extract_action(args);
    if is_fullmoon() {
        match action {
            PamFullmoonAction::Allow => PamResultCode::PAM_SUCCESS,
            PamFullmoonAction::Deny => {
                debug_print("It's fullmoon, so no logins possible", flags);
                PamResultCode::PAM_AUTH_ERR
            }
        }
    } else {
        match action {
            PamFullmoonAction::Allow => {
                debug_print("No fullmoon, no login", flags);
                PamResultCode::PAM_AUTH_ERR
            }
            PamFullmoonAction::Deny => PamResultCode::PAM_SUCCESS,
        }
    }
}

#[no_mangle]
pub extern "C" fn pam_sm_authenticate(
    _: PamHandleT,
    _: PamFlags,
    _: c_int,
    _: *const *const c_char,
) -> PamResultCode {
    PamResultCode::PAM_IGNORE
}

#[no_mangle]
pub extern "C" fn pam_sm_chauthtok(
    _: PamHandleT,
    _: PamFlags,
    _: c_int,
    _: *const *const c_char,
) -> PamResultCode {
    PamResultCode::PAM_IGNORE
}

#[no_mangle]
pub extern "C" fn pam_sm_close_session(
    _: PamHandleT,
    _: PamFlags,
    _: c_int,
    _: *const *const c_char,
) -> PamResultCode {
    PamResultCode::PAM_IGNORE
}

#[no_mangle]
pub extern "C" fn pam_sm_open_session(
    _: PamHandleT,
    _: PamFlags,
    _: c_int,
    _: *const *const c_char,
) -> PamResultCode {
    PamResultCode::PAM_IGNORE
}

#[no_mangle]
pub extern "C" fn pam_sm_setcred(
    _: PamHandleT,
    _: PamFlags,
    _: c_int,
    _: *const *const c_char,
) -> PamResultCode {
    PamResultCode::PAM_IGNORE
}
