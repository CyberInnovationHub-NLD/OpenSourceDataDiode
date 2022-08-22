# 1. Introduction

The Open Source Data Diode (OSDD) is a mid-tier, low-cost, open source data diode aimed at use by public and private parties in the Netherlands. The OSDD consists of a hardware device, the physical diode, and a software suite in which additional functionality can be programmed for specific use cases.

This codebase contains both a prototype version for a proxy framework for data diodes and the hardware schematics of the diode itself. Currently, the OSDD contains a proxy implementation for the Kafka protocol and it also transports metrics (statsd format) through the diode.

All code have been written primarily for Linux systems. 

The OSDD demonstrator was developed in a collaboration between the Ministry of Defense, The Hague Security Delta and Technolution.

## Mission, vision and Objectives
To be added

# 2. Policy
To be added

## Standard for Public Code

We endorse the Standard for Public Code as defined by the Foundation for Public Code. We ask our contributors to endorse this standard as well.
Full information: https://standard.publiccode.net/

## Guidelines for developers
To be added

# 3. Instructions

## Build
Make sure a recent Rust compiler (recently tested with 1.45) and Docker are installed.

You also need MUSL support for RUST: 
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

## Installation
There is a brief installation document available:
[docs/installation.md](docs/installation.md)

## Configuration
There is a brief configuration document available:
[docs/config_file_explanation.md](docs/config_file_explanation.md)

## Design choices
There is a brief design choices document available:
[docs/design_choices.md](docs/design_choices.md)

# 4. License Information

The current license is Apache 2. The code will be published under EUPL. 

Until the formal publication, the original codebase is NOT for distribution and only intended for use by the OSDD community members. 
Branches are free to distribute and publish on discretion of the repective branche owner/moderator.
