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
void init_logger(const char *filter_c_str,
                 const char *write_c_str);

/// Start the microservice and keep exe control until it is complete
///
/// Start the microservice and retain exec until the service exits.
///
/// ```
/// uservice::runService();
/// ```
void runService();

/// Create a health probe
///
/// Create a health probe that can be used to track health of a part of the service and used within a healthcheck to create a readiness or liveness check.
///
/// ```
/// use std::ffi::{CString};
/// let health_name = CString::new("USERVICE_LOG_LEVEL").expect("CString::new failed");
///
/// let hc = uservice::createHealthProbe(health_name.as_ptr(), 2);
/// assert_eq!(hc, 20);
/// ```
int createHealthProbe(const char *name,
                      int margin_ms);

/// Create a call back register function
///
/// This will store the function provided, making it avalable when the callback is to be triggered
int32_t register_callback(void (*callback)(int32_t));

void trigger_callback();

} // extern "C"
