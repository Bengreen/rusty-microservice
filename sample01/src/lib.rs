use log::info;
// use std::ffi::{CStr};
use ffi_log2::{init, LogParam};

use std::process;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Start the microservice and keep exe control until it is complete
///
/// Start the microservice and retain exec until the service exits.
///
/// ```
/// uservice::runService();
/// ```
#[no_mangle]
pub extern "C" fn sample01_run() {
    info!(
        "Initializing the {} {} with PID: {}",
        NAME,
        VERSION,
        process::id()
    );

    info!("Closing {}", NAME);
}

#[no_mangle]
pub extern "C" fn sample01_init_logger_ffi(param: LogParam) {
    init(param);
    info!(
        "Logging registered for {}:{} (PID: {}) using FFI",
        NAME,
        VERSION,
        process::id()
    );
}
