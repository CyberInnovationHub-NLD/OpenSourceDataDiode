## The Open Source Data Diode


## 1. Policy
The Cyber Innovation Hub is included in the Defence Cyber Strategy ([Defensie Cyber Strategie](https://www.defensie.nl/binaries/defensie/documenten/publicaties/2018/11/12/defensie-cyber-strategie-2018/web_Brochure+Defensie+Cyber+Strategie.pdf)) 2018 Dutch Cyber Security Agenda ([Nederlandse Cybersecurity Agenda (NCSA)](https://www.ncsc.nl/onderwerpen/nederlandse-cyber-security-agenda)) 2020. 


### Standard for Public Code
We endorse the Standard for Public Code as defined by the Foundation for Public Code. We ask our contributors to endorse this standard as well.
For the full information, please check https://standard.publiccode.net/

![image](https://user-images.githubusercontent.com/104058636/187181926-5433c767-6fa0-4e04-b89f-4fb818e9a4e0.png)

A video introduction to Standard for Public Code from Creative Commons Global Summit 2020 (4:12) on YouTube.


### Guidelines for developers
To be added


## 2. Introduction 'Open Source Data Diode'
The invention of a data diode is not new. But the fact that the Open-Source Data Diode is offered low-end, low-cost, and as the name suggests, open source is something new. The Open Source Data Diode (OSDD) is a mid-tier, low-cost, open source data diode aimed at use by public and private parties in the Netherlands. 

The currently available data diodes are mainly used for highly classified domains, where a high degree of information security applies. The high demands in highly classified domains make them relatively complex and expensive. However, the OSDD is based on a simple but reliable design and use at low costs, which makes the product more accessible and affordable for a multitude of companies, (semi) governments and individuals. In addition, the basic principle is that the OSDD can be used flexibly.

The OSDD consists of a hardware device, the physical diode, and a software suite in which additional functionality can be programmed for specific use cases. The codebase contains both a prototype version for a proxy framework for data diodes and the hardware schematics of the diode itself. Currently, the OSDD contains a proxy implementation for the Kafka protocol and it also transports metrics (statsd format) through the diode.

All code have been written primarily for Linux systems. 

The OSDD demonstrator was developed in a collaboration between the Cyber Innovation Hub, The Hague Security Delta and Technolution.


### About the Cyber Innovation Hub
The Cyber Innovation Hub, established in 2019 from the Ministry of Defence, ensures that departments, research institutions and companies work together on joint security issues within the field of cyber (security). The aim is to strengthen cyber knowledge and skills in the Netherlands, facilitate innovations and experiments and build an ecosystem of cyber experts, innovators and other partners to reduce cyber threats.

Securing networks is an important point of attention within Cyber. Gaining access to networks by intercepting or modifying network traffic is the basis for many security breeches. One of the problems with securing networks can be traced back to the organization of the current IP-based network traffic. This is bi-directional (2 way traffic), because guaranteed delivery is requested in the protocol. There are various devices or software for securing networks, such as Firewalls, Intruder Detection Systems, Intruder Prevention Systems and data diodes. They all have specific functionality that increases network security. 

A data diode is a device that physically enforces uni-directional (one-way) network traffic. This creates an additional layer of security, as the diode enforces that traffic can only flow from network A to network B, thus protecting network A. For more information about data diodes, see [this document](https://github.com/CyberInnovationHub-NLD/OpenSourceDataDiode-OSDD-/blob/master/General_docs/About%20the%20OSDD/Background%20Information%20about%20the%20OSDD.docx).

![image](https://user-images.githubusercontent.com/104058636/187169728-0fa5b9c2-c291-43c4-81c8-09dcc3c0a1d8.png)


## 3. Instructions

### Build
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

### Result
This build results in two tar files, one for the ingress proxy, one for the egress proxy. Both tar files contain the OSDD service (executable and definition) and a bunch of exported docker containers. 

### Installation
There is a brief installation document available:
[General_docs/installation.md](General_docs/installation.md)

### Configuration
There is a brief configuration document available:
[General_docs/config_file_explanation.md](General_docs/config_file_explanation.md)

### Design choices
There is a brief design choices document available:
[General_docs/design_choices.md](General_docs/design_choices.md)

### Support and/or reporting security issues
In case you need support with the set-up of the OSDD, or if you wish to privately report (security) issues, please contact the maintainer Serina (serina.vandekragt@ictu.nl). When someone lets the maintainer know privately about a security vulnerability, the maintainer develops a fix, validates it, and notifies the developers of the project.

### Report bugs using Github's [issues](https://github.com/CyberInnovationHub-NLD/OpenSourceDataDiode-OSDD-/issues)
We use GitHub issues to track public bugs. Report a bug by opening a new issue.


## 4. Roadmap
The roadmap shows what we are working on and some of the things we have done. The roadmap is only a small guide. It does not cover everything we do, and some things may change. You can contact serina.vandekragt@ictu.nl if you have any questions about the roadmap or suggestions for new features.

### Things we're working on
Now
- Updating the OSDD-repository to meet the Standard of Public Code (and make it as accessible as possible for contributors)
- Engaging with several developing parties to further work on the source code: next up is a workshop to further elaborate on the roadmap and governance file
- Make the OSDD-repository open for everyone to contribute

Next
- Organise a workshop with all stakeholders involved (and the Foundation for Public Code) to form the first use case
- Participate at a hackaton to further develop the source code of the OSDD


## 5. Contributing, authors and acknowledgement
Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are greatly appreciated. 

Get started by reading our [contributors guide](https://github.com/CyberInnovationHub-NLD/OpenSourceDataDiode-OSDD-/blob/master/Contributors%20guide.md).

Please note that this project is released with a [code of conduct](https://github.com/CyberInnovationHub-NLD/OpenSourceDataDiode-OSDD-/blob/master/Code%20of%20Conduct.md). By participating in this project you agree to abide by its terms. 

We’re using Discussions as a place to connect with other members of our community. If you have any questions, great ideas, and/or want to engage with other community members, please leave a message at [Discussions](https://github.com/CyberInnovationHub-NLD/OpenSourceDataDiode-OSDD-/discussions)


## 6. License Information

The current license is Apache 2.

Until the formal publication, the original codebase is NOT for distribution and only intended for use by the OSDD community members. 
Branches are free to distribute and publish on discretion of the repective branche owner/moderator.
