# Minimal K8s App

Create a small runtime that implements the following.
 * [x] CLI parsing and starting
 * [x] readiness/liveness
 * [x] JSON output from readiness/lifeness
 * [ ] YAML config with validation
 * [x] Docker ised build
 * [x] Minimal scratch published container
    * [ ] Follow https://github.com/johnthagen/min-sized-rust
    * [ ] Compare: https://users.rust-lang.org/t/why-does-rust-binary-take-so-much-space/41088/16
    * [x] Implement strip on binary
    * [x] Implement lto on compile
 * [x] respond to k8s lifecycle hooks
 * [x] Prometheus metrics
 * [x] Web service with metrics and logs
 * [x] Benchmark to see/view performance of uService
 * [ ] Kafka support behind a feature control
 * [x] Correctly implement logging so that exec provides the logging implementation and the library references it: https://github.com/rust-lang/log/issues/421
 * [ ] Benchmark difference between function called directly and via ffi callback


# Details Docs/Planning

Cli command to do:

* parse - parse config and validate
* start - start the service and be responsive to readiness/liveness

# Reading list

* Asynchronous across FFI interface https://michael-f-bryan.github.io/rust-ffi-guide/async.html
* FFI Omnibus for Rust http://jakegoulding.com/rust-ffi-omnibus/integers/
* FFI good practice https://spin.atomicobject.com/2013/02/15/ffi-foreign-function-interfaces/
* C API design in Rust https://siliconislandblog.wordpress.com/2019/05/03/lessons-when-creating-a-c-api-from-rust/


# actix-rs

Actix-rs appears to be the bigger better supported web framework in rust but seems to be quite heavyweight in terms of getting it started. It starts with threaded responses and web servers wihtin those. Also when trying to get it started it requires to be run on an async main which may be tricky to map into clap for CLI parsing.
Ideally we can have a synchronous CLI parsing and applicaiton UNTIL such point as we want to dispactch async work and then we start up that on demand.
Ideally we keep the web server light and small for the health system (liveness/readiness/metrics) and then enable larger webserver for actual data traffic. This keeps the footprint of a miminal system down. And keeps optimisations seperated (ie no need to optimise health system if it is lightly used,... also optimisations of data systems implemented wihtout touching health system)

# Warp Http
Using warp to provide http services for liveness/readyness.

# Minimise size of Build
Follow instructions at https://github.com/johnthagen/min-sized-rust
Size for minimal service with liveness, readyness and prometheus metrics = 3.26MB (docker image)

# Add Minimal Web Service
Web service to include minimal serving functions.
Web service to capture prometheus metrics
Web service to write logs

# Run service then test the HUP signal

Run the service then find the PID and send HUP signal

   ./target/debug/uservice start

   kill -HUP <PID>

# Benchmarks

It may be useful to install cargo-criterion as a binary to handle some of the wrapping work for benchmarking

   cargo install cargo-criterion
