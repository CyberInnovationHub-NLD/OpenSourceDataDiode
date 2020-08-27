# Config file
The provided config files are set up to run out of the box. 
This file will describe and explain the diffent entries in the supplied config files.

*Note: Please make sure to use double quotes for all **values** in the config file i.e.* `listening_port = "1234"` *and **NOT*** `listening_port = 1234`

## General Settings
The config file contains some settings that are used by multiple handlers. Those settings are specified under the `[settings]` tag.

#### Settings
* `path` - String, sets the working directory for the docker containers.
* `stats_server` - A string array of hosts and port seperated by a colon. This tells the application where the stats need to be sent to.
* `syslog_host` - IP, a host where the logging of syslog need to be sent.
* `syslog_port` - Integer, a port where the logging of syslog need to be sent.
* `log_level` - String, the amount of logging produced, can be `"Error"`, `"Warn"` `"Info"`, or `"Debug"`
* `instance` - Integer, identifier for this instance of the software
* `network` - String, the side of the data diode, can be `ingress` or `egress`
* `stats_multiplexer_listener_port` - Integer, the port the stats multiplexer is listening on

#### Example
`[settings]`</br>
`path = "/home/osdd"`</br>
`stats_servers = ["10.0.0.1:8125", "127.0.0.1:7654"]`</br>
`syslog_host = "192.168.1.1"`</br>
`syslog_port = "8082"`</br>
`log_level = "Info"`</br>
`instance = "1"`</br>
`network = "ingress"`</br>
`stats_multiplexer_listening_port = "8125"`


## Chain
A chain consists of exactly one transport handler and exactly one protocol handler. A chain can also contain one or more filters. Filters are placed between the protocol handler and the transport handler. Be carefull when adding filters as this can greatly reduce performance. Those settings must be placed under the `[chain.name]` tag where `name` is the name of the chain.

#### Settings
* `protocol_handler` - String, the given protocol handler is added to the chain. The name must match the name given in the handler(see Handler).
* `filter_handlers` - String array, array of all the filters that should be added to the chain. The name must match the name given in the handler(see Handler).
* `transport_handler` - String, the given transport handler is added to the chain. The name must match the name given in the handler(see Handler).

#### Example
`[chain.TestTopic2]`<br>
`protocol_handler = "kafka2"`<br>
`filter_handlers = ["secret_filter"]`<br>
`transport_handler = "udp2"`

## Handler
A handler is a part of the chain. There is one mandatory fields. More fields can be added for more custom commandline arguments. Those settings are under the [protocoltype.name] tag. Where `protocoltype` can be `transporthandler`, `filterhandler` or `protocolhandler` and `name` is the name of the handler(linking to the name given in the Chain).

#### Settings
* `type` - Executable name of the handler.
* optional: `open_udp_port` - Expose the udp port of the docker container. 
* `customfield` - There can be customfield added to the handler

#### Example 
`[protocolhandler.kafka]`<br>
`type = "ph_kafka_ingress"`<br>
`max_bytes_per_partition = "1048576"`<br>
`topic_name = "TestTopic"`<br>
`host_kafka_server = "10.0.0.1"`<br>
`port_kafka_server = "9092"`<br>
`log_level = "Info"`<br>

# Examples of handlers

## UDP Transport Handler
The UDP transport handler is used to transport data over the data diode using UDP. The transport handler on the sending side cuts the data stream into UDP packets. The transport handler on the receiving side combines UDP packets into the original data.

### Ingress

#### Settings

* `type` - `"transport_udp_send"`
* `receiver_address` - IP, the address used by the receiver
* `receiver_port` - Integer, the port used by the receiver
* `sender_address` - IP, the address used by the sender
* `sender_port` - Integer, the port used by the sender
* `bip_buffer_element_count` - Integer, the amount of 1Mb messages that can be buffered
* `send_delay_ms` - Integer, the amount of milliseconds the sender waits before sending the next UDP packet
* `log_level` - String, the amount of logging produced, can be `"Error"`, `"Warn"` `"Info"`, or `"Debug"`

#### Example
`[transporthandler.udp1]`<br>
`type = "transport_udp_send"`<br>
`receiver_address = "192.168.0.255"`<br>
`receiver_port = "1234" `<br>
`sender_address = "192.168.0.255"`<br>
`sender_port = "1234"`<br>
`bip_buffer_element_count = "2"`<br>
`send_delay_ms = "5"`<br>
`log_level = "Info"`<br>

### Egress

#### Settings

* `type` - `"transport_udp_receive"`
* `receiver_address` - String, the address used by the receiver
* `receiver_port` - Integer, the port used by the receiver
* `bip_buffer_element_count` - Integer, the amount of 1Mb messages that can be buffered
* `log_level` - String, the amount of logging produced, can be `"Error"`, `"Warn"` `"Info"`, or `"Debug"`
  

#### Example
`[transporthandler.udp1]`<br>
`type = "transport_udp_receive"`<br>
`receiver_address = "192.168.0.255"`<br>
`receiver_port = "1234"`<br>
`bip_buffer_element_count = "100"`<br>
`log_level = "Info"`

## UDP Handler
The UDP handler sent handles udp packets.

### Ingress

#### Settings
* `listening_port` - Integer, the udp port where the handler listen on.
* `log_level` - String, the amount of logging produced, can be `"Error"`, `"Warn"` `"Info"`, or `"Debug"`
* `bip_buffer_element_count` - Integer, the amount of 1Mb messages that can be buffered

#### Example
`[protocolhandler.stats]`<br>
`type = "ph_udp_ingress"`<br>
`open_udp_port = "7654"`<br>
`listening_port = "7654"`<br>
`log_level = "Info"`<br>
`bip_buffer_element_count = "2"`

### Egress

#### Settings
* `listening_port` - Integer, the udp port where the handler listen on.
* `log_level` - String, the amount of logging produced, can be `"Error"`, `"Warn"` `"Info"`, or `"Debug"`
* `bip_buffer_element_count` - Integer, the amount of 1Mb messages that can be buffered
* `udp_receiver_host` - IP, The host where the udp packets will be sent
* `udp_receiver_port` - Integer, the port where the udp packets will be sent

#### Example
`[protocolhandler.stats]`<br>
`type = "ph_udp_egress"`<br>
`listening_port = "7654"`<br>
`bip_buffer_element_count = "2"`<br>
`udp_receiver_host = "172.17.0.1"`<br>
`udp_receiver_port = "8125"`

## Kafka Handler
The ingress kafka handler is used to fetch data from Kafka. This data is converted into a byte stream. This byte stream is sent to a transport handler.
The kafka egress handler is used to push data to Kafka. The kafka egress handler receives data from a transport handler. This data is pushed into a Kafka server.

### Ingress

#### Settings
* `type` - `"ph_kafka_ingress"` 
* `max_bytes_per_partition` - Max bytes of a messages in kafka
* `topic_name` - String, name of the topic
* `host_kafka_server` - String, the ip address the kafka server is hosted on
* `port_kafka_server` - Integer, the port the kafka server is hosted on
* `log_level` - String, the amount of logging produced, can be `"Error"`, `"Warn"` `"Info"`, or `"Debug"`
* `bip_buffer_element_count` - Integer, the amount of 1Mb messages that can be buffered


#### Example
`[protocolhandler.kafka]`<br>
`type = "ph_kafka_ingress"`<br>
`max_bytes_per_partition = "1048576"`<br>
`topic_name = "TestTopic"`<br>
`host_kafka_server = "10.0.0.1"`<br>
`port_kafka_server = "9092"`<br>
`log_level = "Info`<br>
`bip_buffer_element_count = "2"`

### Egress

#### Settings
* `type` - `"ph_kafka_egress"`
* `host_kafka_server` - String, the IP address the kafka server is hosted on
* `port_kafka_server` - Integer, the port the kafka server is hosted on
* `in_replacement` - It replaces the given topic name with the name given in `out_replacement`. 
* `out_replacement` - See in_replacement
* `bip_buffer_element_count` - Integer, the amount of 1Mb messages that can be buffered
* `log_level` - String, the amount of logging produced, can be `"Error"`, `"Warn"` `"Info"`, or `"Debug"`

#### Example
`[protocolhandler.kafka]`<br>
`type = "ph_kafka_egress"`<br>
`host_kafka_server = "10.0.0.2"`<br>
`port_kafka_server = "9092"`<br>
`bip_buffer_element_count = "10"`<br>
`in_replacement = "TestTopic"`<br>
`out_replacement = "TestTopic2"`<br>
`log_level = "Info"`<br>

## Filter Handler Strings
The filter handler can be used to scan the first bytes of a byte array for a given word. Every message containing this word is dropped by the filter. The filter is placed between a handler and a transport handler or a transport handler and another filter.

#### Settings
* `type` - String, the handler type. `type` can be `"filter"`
* `bip_buffer_element_count` - usize, the amount of 1Mb messages that can be buffered
* `word_to_filter` - String, the handler filters the name
* `log_level` - String, the amount of logging produced, can be `"Error"`, `"Warn"` `"Info"`, or `"Debug"`


#### Example
`[filterhandler.secret_filter]`<br>
`type = "filter"`<br>
`log_level = "Info"`<br>
`word_to_filter = "SECRET"`<br>
`bip_buffer_element_count = "2"`