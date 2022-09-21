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

use crate::errors::ErrorKind::*;
use crate::errors::*;
use crate::*;

/// All configurations read form the toml file.
pub struct TomlConfig {
    pub chains: Vec<Chain>,
    pub handlers: Vec<Handler>,
    pub settings: Settings,
}

#[derive(Debug, Deserialize)]
struct ChainToml {
    pub protocol_handler: String,
    pub filter_handlers: Vec<String>,
    pub transport_handler: String,
}

/// Convert TOML file to settings, chains and handlers.
/// println errors because logging is not initialized yet.
pub fn read_toml(config_file: &str) -> Result<TomlConfig> {
    let mut handlers: Vec<Handler> = Vec::new();
    let mut chains: Vec<Chain> = Vec::new();
    let mut settings_option: Option<Settings> = None;
    let toml_string = fs::read_to_string(config_file)
        .chain_err(|| ConfigurationError("Config nog found".to_string()))?;
    let toml_value: Value = toml::from_str(&toml_string)
        .chain_err(|| ConfigurationError("Cannot convert config to toml".to_string()))?;

    if let Some(tables) = toml_value.as_table() {
        for table in tables {
            //check key of each table
            match table.0.as_ref() {
                "settings" => {
                    settings_option = table.1.clone().try_into().ok();
                }
                "chain" => {
                    if let Some(chain_list) = table.1.as_table() {
                        for chain_toml in chain_list {
                            let chain_struct: ChainToml =
                                chain_toml.1.clone().try_into().chain_err(|| {
                                    ConfigurationError(format!(
                                        "Chain {} in wrong format",
                                        chain_toml.0
                                    ))
                                })?;
                            let chain_with_name = Chain {
                                name: chain_toml.0.to_string(),
                                protocol_handler: chain_struct.protocol_handler,
                                filter_handlers: chain_struct.filter_handlers,
                                transport_handler: chain_struct.transport_handler,
                            };

                            chains.push(chain_with_name);
                        }
                    }
                }
                "protocolhandler" => {
                    if let Some(protocol_handlers) = table.1.as_table() {
                        for handler in protocol_handlers {
                            handlers.push(read_handler(handler, HandlerType::ProtocolHandler)?);
                        }
                    }
                }
                "filterhandler" => {
                    if let Some(filter_handlers) = table.1.as_table() {
                        for filter in filter_handlers {
                            handlers.push(read_handler(filter, HandlerType::FilterHandler)?);
                        }
                    }
                }
                "transporthandler" => {
                    if let Some(transport_handlers) = table.1.as_table() {
                        for transport in transport_handlers {
                            handlers.push(read_handler(transport, HandlerType::TransportHandler)?);
                        }
                    }
                }
                _ => log::warn!("in config {} is unknown!", table.0),
            }
        }
    };
    match settings_option {
        Some(settings) => Ok(TomlConfig {
            chains,
            handlers,
            settings,
        }),
        None => Err(
            ConfigurationError("Settings not found in the configuration file".to_string()).into(),
        ),
    }
}

/// Convert a handler from a TOML file to a handler struct.
/// `type` and `open_udp_port` are specials cases and needed in the configuration in osdd.
/// All other arguments are store in a vec and given as argument to the executable.
/// `type` is the executabe and docker name
/// `open_udp_port` is to open een udp port in the docker container
fn read_handler(handler_config: (&String, &Value), handler_type: HandlerType) -> Result<Handler> {
    let mut executable_option: Option<&str> = None;
    let mut udp_port_option: Option<u16> = None;
    let mut arguments = Vec::new();

    //read arguments from the handler_config.
    if let Some(toml_arguments) = handler_config.1.as_table() {
        for argument in toml_arguments {
            if let Some(x) = argument.1.as_str() {
                match argument.0.as_ref() {
                    //Type is to define which type of handler it is
                    "type" => {
                        executable_option = argument.1.as_str();
                    }
                    "open_udp_port" => {
                        udp_port_option = match argument.1.as_str() {
                            Some(x) => Some(match x.parse::<u16>() {
                                Ok(v) => v,
                                Err(_) => {
                                    return Err(ConfigurationError(format!(
                                        "Cannot parse open udp port to a port in {}",
                                        handler_config.0
                                    ))
                                    .into())
                                }
                            }),
                            None => {
                                return Err(ConfigurationError(format!(
                                    "Cannot parse open udp port to a port in {}",
                                    handler_config.0
                                ))
                                .into())
                            }
                        }
                    }
                    //All other arguments are arguments for the handler
                    _ => arguments.push((argument.0.to_string(), x.to_string())),
                }
            }
        }
    }
    if let Some(executable) = executable_option {
        Ok(Handler {
            name: handler_config.0.to_string(),
            executable: executable.to_string(),
            arguments,
            handler_type,
            incoming_socket: None,
            outgoing_socket: None,
            udp_port_option,
        })
    } else {
        Err(ConfigurationError(format!(
            "Cannot read the the type of handler {}",
            handler_config.0
        ))
        .into())
    }
}
