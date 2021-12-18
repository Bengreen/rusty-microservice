#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

extern "C" {

/// Initialize the logger
///
/// The logger env_logger is built into the so and is initialised using this function. BUT this is not quite right.
/// The logger should not have to be implemented inside the so. It should be possible to implement the logger in the exe and not the so.
/// The so role is to use the log methods and not have to implement the log backend.
///
/// ```
/// use std::ffi::{CString};
/// let log_env = CString::new("USERVICE_LOG_LEVEL").expect("CString::new failed");
/// let write_env = CString::new("USERVICE_WRITE_STYLE").expect("CString::new failed");
///
/// unsafe{uservice::init_logger(log_env.as_ptr(), write_env.as_ptr());}
/// ```
void sample01_init_logger(const char *filter_c_str,
                          const char *write_c_str);

/// Start the microservice and keep exe control until it is complete
///
/// Start the microservice and retain exec until the service exits.
///
/// ```
/// uservice::runService();
/// ```
void sample01_run();

} // extern "C"
