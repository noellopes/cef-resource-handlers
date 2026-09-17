use anyhow::{Result, anyhow};
use cef::*;

pub fn initialize_cef(main_args: &MainArgs, app: &mut App, sandbox_info: *mut u8) -> Result<()> {
    let settings = Settings {
        no_sandbox: sandbox_info.is_null() as _,
        ..Default::default()
    };

    match initialize(Some(main_args), Some(&settings), Some(app), sandbox_info) {
        1 => Ok(()), // true
        code => Err(anyhow!("Failed to initialize CEF with code: {code}")),
    }
}
