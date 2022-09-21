# 1. About the Cyber Innovation Hub
The Cyber Innovation Hub, established in 2019 from the Ministry of Defence, ensures that departments, research institutions and companies work together on joint security issues within the field of cyber (security). The aim is to strengthen cyber knowledge and skills in the Netherlands, facilitate innovations and experiments and build an ecosystem of cyber experts, innovators and other partners to reduce cyber threats.

# 2. Introduction 'Open Source Data Diode'
The Open Source Data Diode (OSDD) is a mid-tier, low-cost, open source data diode aimed at use by public and private parties in the Netherlands. The OSDD consists of a hardware device, the physical diode, and a software suite in which additional functionality can be programmed for specific use cases.

This codebase contains both a prototype version for a proxy framework for data diodes and the hardware schematics of the diode itself. Currently, the OSDD contains a proxy implementation for the Kafka protocol and it also transports metrics (statsd format) through the diode.

All code have been written primarily for Linux systems. 

The OSDD demonstrator was developed in a collaboration between the Ministry of Defense, The Hague Security Delta and Technolution.

![image](https://user-images.githubusercontent.com/104058636/187169728-0fa5b9c2-c291-43c4-81c8-09dcc3c0a1d8.png)

# 3. Policy
The Cyber Innovation Hub is included in the Defence Cyber Strategy ([Defensie Cyber Strategie](https://www.defensie.nl/binaries/defensie/documenten/publicaties/2018/11/12/defensie-cyber-strategie-2018/web_Brochure+Defensie+Cyber+Strategie.pdf)) 2018 Dutch Cyber Security Agenda ([Nederlandse Cybersecurity Agenda (NCSA)](https://www.ncsc.nl/onderwerpen/nederlandse-cyber-security-agenda)) 2020. 

## Standard for Public Code
We endorse the Standard for Public Code as defined by the Foundation for Public Code. We ask our contributors to endorse this standard as well.
For the full information, please check https://standard.publiccode.net/

![image](https://user-images.githubusercontent.com/104058636/187181926-5433c767-6fa0-4e04-b89f-4fb818e9a4e0.png)

A video introduction to Standard for Public Code from Creative Commons Global Summit 2020 (4:12) on YouTube.


## Guidelines for developers
To be added

# 4. Instructions

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
[General_docs/installation.md](General_docs/installation.md)

## Configuration
There is a brief configuration document available:
[General_docs/config_file_explanation.md](General_docs/config_file_explanation.md)

## Design choices
There is a brief design choices document available:
[General_docs/design_choices.md](General_docs/design_choices.md)

## Support
In case you need support with the set-up of the OSDD, please contact Serina (serina.vandekragt@ictu.nl).

## Roadmap
To be added

# 5. Contributing, authors and acknowledgement
Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are greatly appreciated. 

Get started by reading our [contributors guide](https://github.com/CyberInnovationHub-NLD/OpenSourceDataDiode-OSDD-/blob/master/Contributors%20guide.md).

Please note that this project is released with a [code of conduct](https://github.com/CyberInnovationHub-NLD/OpenSourceDataDiode-OSDD-/blob/master/Code%20of%20Conduct.md). By participating in this project you agree to abide by its terms. 

# 6. License Information

The current license is Apache 2. The code will be published under EUPL. 

Until the formal publication, the original codebase is NOT for distribution and only intended for use by the OSDD community members. 
Branches are free to distribute and publish on discretion of the repective branche owner/moderator.
