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
void sample01_run();

void init_logger(LogParam param);

const char *name();

const char *version();

} // extern "C"
