#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

extern "C" {

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
int32_t register_callback(int32_t (*callback)(int32_t));

void trigger_callback();

void uservice_init_logger_ffi(LogParam param);

} // extern "C"
