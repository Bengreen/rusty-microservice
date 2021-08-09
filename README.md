# Minimal K8s App


Create a small runtime that implements the following.
 * [x] CLI parsing and starting
 * [o] Change warp for actix-web (Actix seems to not be easily mappable to single thread and initialised from non-async code)
 * [ ] readiness/liveness
 * [ ] JSON output from readiness/lifeness
 * [ ] YAML config with validation
 * [ ] Docker ised build
 * [ ] Minimal scratch published container
 * [ ] respond to k8s lifecycle hooks
 * [ ] Prometheus metrics


# Details Docs/Planning

Cli command to do:

* parse - parse config and validate
* start - start the service and be responsive to readiness/liveness

# actix-rs

Actix-rs appears to be the bigger better supported web framework in rust but seems to be quite heavyweight in terms of getting it started. It starts with threaded responses and web servers wihtin those. Also when trying to get it started it requires to be run on an async main which may be tricky to map into clap for CLI parsing.
Ideally we can have a synchronous CLI parsing and applicaiton UNTIL such point as we want to dispactch async work and then we start up that on demand.
Ideally we keep the web server light and small for the health system (liveness/readiness/metrics) and then enable larger webserver for actual data traffic. This keeps the footprint of a miminal system down. And keeps optimisations seperated (ie no need to optimise health system if it is lightly used,... also optimisations of data systems implemented wihtout touching health system)

