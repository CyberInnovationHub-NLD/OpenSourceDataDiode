# Manual Installation
In order to run the provided software, you will need the following:

* Ubuntu Server 18.04 LTS image
* Installation medium
* Two machines
* An active internet connection
* The ingress.tar.gz file
* The egress.tar.gz file

## Installing Ubuntu 18.04 LTS
The first step is to download Ubuntu 18.04 LTS. Create a bootable usb or cd of this Ubuntu release. Next, install Ubuntu 18.04 LTS on the two proxy machines.

## Installing Docker
The software uses Docker to start the different parts of the application in seperate containers. Install docker using apt:
`sudo apt-get install docker.io`

Tell Docker to start when the computer boots using the following command:
`sudo systemctl enable docker`

## Configuring Users
The OSDD service is configured to run under the `osdd` user. For security purposes, this user has no root rights. This user will need to be created. This process needs to be repeated for both machines. The OSDD user needs to be a member of the docker group.

## Installing OSDD
The application makes use of a couple of docker images and some configuration files. These files need to be installed in the right location in order for the application to work correctly. The process below needs to be repeated for both proxies. The files need to be configured in the following way:

* Extract the supplied `.tar.gz` to `/tmp` with `tar -C /tmp -xzvf osdd_ingress.tar.gz --transform='s/.*\///'`
* Load Docker containers from the extracted tar.gz file (see https://docs.docker.com/engine/reference/commandline/import/)
* Copy `osdd` to the `/home/osdd` folder
* Copy `Config.toml` to the `/home/osdd` folder.
* Copy `osdd.service` to the `/etc/systemd/system` folder
* Give `chmod +x` permissions to the executable files. 
* Reload the service units with `systemctl daemon-reload`
* Start the osdd service `sudo systemctl start osdd`
* Enable the osdd service to run at startup(optional): 
  `sudo systemctl enable osdd`

## Configuring IP addresses
In order for the software to run, the correct ip addresses need to be configured. With the standard config file the following ip addresses should be given to each machine:

#### Ingress Proxy
* Connection to the kafka-server-tx - 10.0.0.2
* Connection to the IN of the data diode 192.168.0.1

#### Egress Proxy
* Connection to the kafka-server-rx - 10.0.0.1
* Connection to the OUT of the data diode 192.168.0.2

#### Kafka tx
* Connection to the ingress proxy - 10.0.0.1

#### Kafka rx
* Connection to the egress proxy - 10.0.0.2

## Installing Speedometer(optional)
Speedometer is a tool that can be used to view statistics on the networkcard of the computer. It is installed from apt using the following command:
`sudo apt-get install speedometer`

## Installing Trafshow(optional)
Trafshow is a tool like Speedometer. It can give a more detailed insight into the data sent and received. It can be installed from apt using the following command: 
`sudo apt-get install netdiag`

## Installing Ethtool(optional)
Ethtool can be used to change the speed of a network interface. This can be used to test with a different configuration. Ethtool can be installed with the following command: 
`sudo apt-get install ethtool`

## Installing CollectD(optional)
CollectD can be used to collect various statistics about the computer it is running on. It can be used to monitor the proxies.
CollectD can be installed with the following command: 
`sudo apt-get install collectd collectd-utils`




