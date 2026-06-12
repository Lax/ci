use std::sync::Mutex;

use anyhow::Result;

use crate::config::Config;

static INIT: Mutex<bool> = Mutex::new(false);

/// Return a reference to the librime RimeApi struct.
fn get_api<'a>() -> Result<&'a librime_sys::RimeApi> {
    let ptr = unsafe { librime_sys::rime_get_api() };
    if ptr.is_null() {
        anyhow::bail!("rime_get_api returned null (librime not loaded)");
    }
    Ok(unsafe { &*ptr })
}

/// Ensure librime is initialized. Safe to call multiple times.
pub fn ensure_initialized(cfg: &Config) -> Result<()> {
    let mut init = INIT.lock().unwrap();
    if *init {
        return Ok(());
    }
    try_initialize(cfg)?;
    *init = true;
    Ok(())
}

fn try_initialize(cfg: &Config) -> Result<()> {
    let api = get_api()?;

    // Check if already initialized by another process (e.g. Fcitx5)
    let sid = unsafe { (api.create_session.unwrap())() };
    if sid != 0 {
        unsafe { (api.destroy_session.unwrap())(sid) };
        let mut init = INIT.lock().unwrap();
        *init = true;
        return Ok(());
    }

    let user_dir = cfg.rime_user_dir();
    let mut traits = rime_api::Traits::new();
    traits.set_user_data_dir(&user_dir);
    traits.set_distribution_name("ci");
    traits.set_distribution_code_name("ci");
    traits.set_distribution_version("0.1.0");

    rime_api::setup(&mut traits);
    rime_api::initialize(&mut traits);

    log::info!("librime initialized (user_dir={user_dir})");
    Ok(())
}

/// Call librime sync_user_data() directly via FFI.
/// Returns Err if unavailable (caller should fall back to rime_dict_manager).
pub fn sync_user_data(cfg: &Config) -> Result<()> {
    ensure_initialized(cfg)?;

    let api = get_api()?;
    let func = api.sync_user_data;
    if func.is_none() {
        anyhow::bail!("sync_user_data not available in this librime version");
    }
    let ok = unsafe { func.unwrap()() };
    if ok == 0 {
        anyhow::bail!("librime sync_user_data returned false");
    }
    log::info!("librime sync_user_data completed");
    Ok(())
}
