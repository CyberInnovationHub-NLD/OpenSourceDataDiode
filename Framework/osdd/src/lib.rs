// Copyright 2020 Ministerie van Defensie
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Starts processes from the given commands
pub mod docker_runner;
/// Error chain for OSDD
pub mod errors;
/// Read configuration out of the toml file
pub mod read_toml;
/// UDP multiplexer for statitics
pub mod udp_multiplexer_stats;
use crate::errors::ErrorKind::ConfigurationError;
use crate::errors::*;
use serde::Deserialize;
use std::fs;
use std::process::Command;
use toml::Value;

const PATH_PREFIX_UNIX_SOCKETS_ON_PROXY: &str = "/home/osdd/sockets/";
const PATH_PREFIX_UNIX_SOCKETS_IN_DOCKER: &str = "/tmp/";

/// Set from socket port to `0` for syslog. (0 is auto assiging to a port)
pub const PORT_FROM_UDP_SYSLOG: u16 = 0;
/// Set from socket host to `0.0.0.0` for syslog
pub const FROM_HOST_UDP_SYSLOG: &str = "0.0.0.0";
/// The maximum size of the packet buffer.
/// The field size sets a theoretical limit of 65,535 bytes (8 byte header + 65,527 bytes of data) for a UDP datagram.\
/// However the actual limit for the data length, which is imposed by the underlying IPv4 protocol, is 65,507 bytes (65,535 − 8 byte UDP header − 20 byte IP header).
pub const MAX_BUFFER_SIZE_BYTES: usize = 65507;

/// OSDD Settings
#[derive(Debug, Deserialize)]
pub struct Settings {
    /// Sets the working directory for the docker containers.
    pub path: String,
    /// A string array of socket addresses. This tells the application where the stats need to be sent to.
    pub stats_servers: Vec<String>,
    /// Tells where the logging of syslog need to be sent.
    pub syslog_host: String,
    /// Tells where the logging of syslog need to be sent.
    pub syslog_port: String,
    /// The amount of logging produced, can be "Error", "Warn" "Info", or "Debug"
    pub log_level: String,
    /// Identifier for this instance of the software
    pub instance: String,
    /// The side of the data diode, can be ingress or egress
    pub network: String,
    /// The port the stats multiplexer is listening on
    pub stats_multiplexer_listening_port: String,
}

/// A chain consists of exactly one transport handler and exactly one protocol handler. A chain can also contain one or more filters. Filters are placed between the protocol handler and the transport handler.
pub struct Chain {
    pub name: String,
    ///The name of the protocol handler must match the name given in the handler
    pub protocol_handler: String,
    ///The name of the filter handler must match the name given in the handler
    pub filter_handlers: Vec<String>,
    ///The name of the transport handler must match the name given in the handler
    pub transport_handler: String,
}

///A handler read from the TOML file
#[derive(Debug)]
pub struct Handler {
    name: String,
    executable: String,
    arguments: Vec<(String, String)>,
    handler_type: HandlerType,
    incoming_socket: Option<String>,
    outgoing_socket: Option<String>,
    udp_port_option: Option<u16>,
}

#[derive(PartialEq, Debug)]
enum HandlerType {
    TransportHandler,
    FilterHandler,
    ProtocolHandler,
}

///A command with a name
#[derive(Debug)]
pub struct CommandWithName {
    pub command: Command,
    pub name: String,
}

impl Handler {
    /// Create Docker command from a handler
    fn create_command(
        &self,
        chain_name: &str,
        stats_port: u16,
        settings: &Settings,
    ) -> Result<CommandWithName> {
        let mut command = Command::new("docker");
        command.args(&["run"]);
        command.args(&["-d"]);

        if self.handler_type == HandlerType::TransportHandler {
            command.args(&["--network", "host"]);
            command.args(&["--cap-add=sys_nice"]);
        }

        //if "open_udp_port" is given to the handler then publish on the same port
        if let Some(port) = self.udp_port_option {
            command.args(&[format!("--publish={}:{}/udp", port, port)]);
        }

        //short name is needed for the correct naming format
        let handler_type_short_name = match self.handler_type {
            HandlerType::TransportHandler => "transport",
            HandlerType::FilterHandler => "filter",
            HandlerType::ProtocolHandler => "ph",
        };

        let chain_handler_name = format!(
            "osdd.{}.{}.{}.{}.{}",
            &settings.instance, &settings.network, chain_name, handler_type_short_name, &self.name
        );

        command.args(&["--name", &chain_handler_name]);

        //mount sockets path
        command.args(&[
            "--mount",
            &format!(
                "type=bind,source={},target={}",
                PATH_PREFIX_UNIX_SOCKETS_ON_PROXY, PATH_PREFIX_UNIX_SOCKETS_IN_DOCKER
            ),
        ]);

        //Removes the Container after stopping
        command.args(&["--restart", "always"]);

        command.args(&["--entrypoint", &format!("./{}", &self.executable)]);

        command.args(&[&self.executable]);
        //Load all arguments
        for argument in &self.arguments {
            let dash_dash_argument = format!("--{}", argument.0);
            command.args(&[dash_dash_argument, argument.1.to_string()]);
        }

        //Arguments for sockets
        match self.handler_type {
            HandlerType::ProtocolHandler | HandlerType::TransportHandler => {
                command_socket_path_transport_protocol(&self, &mut command)?
            }
            HandlerType::FilterHandler => command_socket_paths_filter(&self, &mut command)?,
        };

        //Set standard arguments
        if self.handler_type == HandlerType::ProtocolHandler
            || self.handler_type == HandlerType::FilterHandler
        {
            command.args(&["--stats_server_address", "172.17.0.1"]);
        } else {
            command.args(&["--stats_server_address", "127.0.0.1"]);
        }
        command.args(&["--stats_server_port", &stats_port.to_string()]);
        command.args(&["--from_host_sys_log", FROM_HOST_UDP_SYSLOG]);
        command.args(&["--from_port_sys_log", &PORT_FROM_UDP_SYSLOG.to_string()]);
        command.args(&["--to_host_sys_log", &settings.syslog_host]);
        command.args(&["--to_port_sys_log", &settings.syslog_port]);
        command.args(&["--handler_name", &chain_handler_name]);

        command.current_dir(settings.path.to_string());

        Ok(CommandWithName {
            command,
            name: chain_handler_name,
        })
    }
}

/// Creates docker commands of the given handlers
pub fn create_commands_all_handlers(
    chains: Vec<Chain>,
    mut handlers_config: Vec<Handler>,
    stats_multiplexer_listening_port_u16: u16,
    settings: &Settings,
) -> Result<Vec<CommandWithName>> {
    let mut commands: Vec<CommandWithName> = Vec::new();
    for mut chain in chains {
        //Create a vector of the chain
        let mut handlers_to_create: Vec<String> = Vec::new();
        handlers_to_create.push(chain.protocol_handler);
        handlers_to_create.append(&mut chain.filter_handlers);
        handlers_to_create.push(chain.transport_handler);

        //loop all pairs of the chain
        //Set outgoing socket for the first and incoming socket fot the second
        for item in handlers_to_create.windows(2) {
            let process_to_start1 = &item[0];
            let process_to_start2 = &item[1];
            assign_sockets(
                &mut handlers_config,
                process_to_start1,
                process_to_start2,
                &chain.name,
            )?;
        }

        //Create commands to run dockers with all settings get and set before
        for handler_to_create in handlers_to_create {
            match handlers_config.iter().find(|x| x.name == handler_to_create) {
                Some(handler_config) => commands.push(handler_config.create_command(
                    &chain.name,
                    stats_multiplexer_listening_port_u16,
                    &settings,
                )?),
                None => {
                    return Err(ConfigurationError(format!(
                        "Cannot find {} as handler in config",
                        handler_to_create
                    ))
                    .into())
                }
            }
        }
    }
    Ok(commands)
}

fn assign_sockets(
    handlers_config: &mut Vec<Handler>,
    process1: &str,
    process2: &str,
    chain_name: &str,
) -> Result<()> {
    match handlers_config.iter_mut().find(|x| x.name == process1) {
        Some(handler) => {
            handler.outgoing_socket = Some(format!(
                "{}{}_{}_{}",
                PATH_PREFIX_UNIX_SOCKETS_IN_DOCKER, chain_name, process1, process2
            ))
        }
        None => {
            return Err(ConfigurationError(format!(
                "Cannot find {} as handler in config",
                process1
            ))
            .into())
        }
    };

    match handlers_config.iter_mut().find(|x| x.name == process2) {
        Some(handler) => {
            handler.incoming_socket = Some(format!(
                "{}{}_{}_{}",
                PATH_PREFIX_UNIX_SOCKETS_IN_DOCKER, chain_name, process1, process2
            ))
        }
        None => {
            return Err(ConfigurationError(format!(
                "Cannot find {} as handler in config",
                process2
            ))
            .into())
        }
    };

    Ok(())
}

fn command_socket_path_transport_protocol(handler: &Handler, command: &mut Command) -> Result<()> {
    let socket_path = match &handler.incoming_socket {
        Some(x) => x,
        None => &handler.outgoing_socket.as_ref().chain_err(|| {
            ConfigurationError(format!(
                "Cannot bind {} to other handler in chain",
                handler.name
            ))
        })?,
    };
    command.args(&["--socket_path", &socket_path]);
    Ok(())
}
fn command_socket_paths_filter(handler: &Handler, command: &mut Command) -> Result<()> {
    let incoming_socket = handler.incoming_socket.as_ref().chain_err(|| {
        ConfigurationError(format!(
            "Cannot bind {} to other handler in chain",
            handler.name
        ))
    })?;
    command.args(&["--socket_path_in", &incoming_socket]);
    let outgoing_socket = handler.outgoing_socket.as_ref().chain_err(|| {
        ConfigurationError(format!(
            "Cannot bind {} to other handler in chain",
            handler.name
        ))
    })?;
    command.args(&["--socket_path_out", &outgoing_socket]);
    Ok(())
}
