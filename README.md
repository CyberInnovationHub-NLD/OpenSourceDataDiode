# Introduction

This codebase contains both a prototype version for a proxy framework for data diodes and the hardware schematics of the diode itself. Currently the OSDD contains a proxy implementation for the Kafka protocol and it also transports metrics (statsd format) through the diode.

All code has been written primarily for Linux systems. 

# License Information

Currently license is Apache 2. Code will be published under EUPL. 

Untill formal publication code is not for distribution. 

# Build
Make sure a recent Rust compiler (recently tested with 1.45) and Docker are installed.

You also needs MUSL support for RUST: 
```sh
rustup target add x86_64-unknown-linux-musl
```

Run the build script in the scripts folder:
```sh
cd scripts
./create_tars.sh
```

This scripts builds the repository in release and in MUSL release, it create docker images and create two tars. One for the ingress proxy and one for the egress proxy.

Copy the tars to your target systems, unpack them and install the *osdd* service. Match the configuration file to your system and start the osdd service.

## Result
This build results in two tar files, one for the ingress proxy, one for the egress proxy. Both tar files contain the OSDD service (executable and definition) and a bunch of exported docker containers. 

# Installation
There is a brief installation document available:
[docs/installation.md](docs/installation.md)

# Configuration
There is a brief configuration document available:
[docs/config_file_explanation.md](docs/config_file_explanation.md)

# Design choices
There is a brief design choices document available:
[docs/design_choices.md](docs/design_choices.md)
