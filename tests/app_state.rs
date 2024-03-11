#![allow(clippy::missing_safety_doc)]

use mangadex_desktop_api2::{AppState, ManagerCoreResult};
use once_cell::sync::OnceCell;

static mut APP_STATE: OnceCell<AppState> = OnceCell::new();

async fn init() -> ManagerCoreResult<AppState> {
    AppState::init().await
}

async unsafe fn set() -> ManagerCoreResult<()> {
    let _ = APP_STATE.set(init().await?);
    Ok(())
}

pub async unsafe fn get_mut() -> ManagerCoreResult<&'static mut AppState> {
    if let Some(app) = APP_STATE.get_mut() {
        Ok(app)
    } else {
        set().await?;
        Ok(APP_STATE
            .get_mut()
            .ok_or(anyhow::Error::msg("The Test App State cannot be loaded"))?)
    }
}
