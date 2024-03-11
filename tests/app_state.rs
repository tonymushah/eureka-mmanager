use mangadex_desktop_api2::{AppState, ManagerCoreResult};
use once_cell::sync::OnceCell;

static mut APP_STATE: OnceCell<AppState> = OnceCell::new();

fn init() -> ManagerCoreResult<AppState> {
    tokio::runtime::Handle::current().block_on(async { AppState::init().await })
}

unsafe fn set() -> ManagerCoreResult<()> {
    let _ = APP_STATE.set(init()?);
    Ok(())
}

pub unsafe fn get_mut() -> ManagerCoreResult<&'static mut AppState> {
    if let Some(app) = APP_STATE.get_mut() {
        Ok(app)
    } else {
        set()?;
        Ok(APP_STATE
            .get_mut()
            .ok_or(anyhow::Error::msg("The Test App State cannot be loaded"))?)
    }
}
