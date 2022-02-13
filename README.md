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
  * [x] Docker build for cargo: https://github.com/LukeMathWalker/cargo-chef


# Details Docs/Planning

Cli command to do:

* parse - parse config and validate
* start - start the service and be responsive to readiness/liveness

# Reading list

  * Asynchronous across FFI interface https://michael-f-bryan.github.io/rust-ffi-guide/async.html
  * FFI Omnibus for Rust http://jakegoulding.com/rust-ffi-omnibus/integers/
  * FFI good practice https://spin.atomicobject.com/2013/02/15/ffi-foreign-function-interfaces/
  * FFI for fun and profit https://michael-f-bryan.github.io/rust-ffi-guide/overview.html
  * Passing strings through to FFI https://rust-unofficial.github.io/patterns/idioms/ffi/passing-strings.html
  * C API design in Rust https://siliconislandblog.wordpress.com/2019/05/03/lessons-when-creating-a-c-api-from-rust/
  * Multi arch Docker build: https://www.docker.com/blog/multi-arch-images/
  * Singletons... if you must use them http://oostens.me/posts/singletons-in-rust/

# Useful tools

* https://play.rust-lang.org/

# Development Setup

When setting up for development you need to provide a path for the shared lib.
On OSX use:

    export DYLD_LIBRARY_PATH=${PWD}/target/debug/deps

    cargo run run -- -l libuservice.dylib start

or call it directly

    cargo run -- -l target/debug/deps/libuservice.dylib start

# Setup test for k8s

Setup k8s cluster and deploy sample application to it

Connect to gcloud shell

    gcloud cloud-shell ssh --authorize-session

Create k8s cluster

```bash
export PROJECT=istiotest-285618
export CLUSTER=test0
export REGION=us-central1
gcloud container --project "${PROJECT}" clusters create-auto "${CLUSTER}" --region "${REGION}" --release-channel "regular" --network "projects/${PROJECT}/global/networks/default" --subnetwork "projects/${PROJECT}/regions/${REGION}/subnetworks/default"
# --cluster-ipv4-cidr "/17" --services-ipv4-cidr "/22"
```

Connect to cluster


    gcloud container clusters get-credentials ${CLUSTER} --region ${REGION} --project ${PROJECT}

Setup VSCode for gcloud:
* https://medium.com/@alex.burdenko/vs-code-happens-to-be-my-favorite-code-editor-and-ive-been-lucky-to-participate-so-many-diverse-952102856a7a

Use gcloud command to find the hostname of the cloud server we want to connect to

    gcloud cloud-shell ssh --dry-run

Then get hostname and write the hostname to the ssh config.

# Delete the cluster

    gcloud container --project "${PROJECT}" clusters delete "${CLUSTER}" --region "${REGION}"

# Docker Developer testing

Pull down the docker image that has been build using google

    docker pull us-docker.pkg.dev/istiotest-285618/rusty-microservice/rusty

    docker run -it us-docker.pkg.dev/istiotest-285618/rusty-microservice/rusty:latest

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
