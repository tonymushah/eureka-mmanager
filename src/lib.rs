pub use crate::r#core::ManagerCoreResult;

pub mod settings;

use crate::settings::verifications::{
    data::{initialise_data_dir, verify_data_dir},
    settings::{initialise_settings_dir, verify_settings_dir},
};
use log::{info, warn};
mod r#core;

pub use crate::r#core::{Error, ErrorType};

pub mod r#static;

/// Verify if the data dir and the settings are all there
/// if on of them are not defined or not found , it automatically create the dir corresponding to the error
pub fn verify_all_fs() -> std::io::Result<()> {
    match verify_settings_dir() {
        Ok(data) => {
            info!("{}", data);
        }
        Err(error) => {
            warn!("{}", error);
            warn!("Settings dir not found ");
            info!("Initializing...");
            match initialise_settings_dir() {
                Ok(data) => data,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ));
                }
            };
        }
    };
    info!("Initilized settings dir !");
    match verify_data_dir() {
        Ok(data) => {
            info!("{}", data);
        }
        Err(error) => {
            warn!("{}", error);
            warn!("Data dir not found \n");
            info!("\tInitializing...");
            match initialise_data_dir() {
                Ok(data) => data,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ));
                }
            };
        }
    }
    Ok(())
}
