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

#![allow(clippy::large_enum_variant)]
use error_chain::*;
use std::process::Command;

pub trait ErrorChainPanicUnwrap<T> {
    fn chain_unwrap(self) -> T;
}

impl<T> ErrorChainPanicUnwrap<T> for Result<T> {
    fn chain_unwrap(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => panic!("{}", e.display_chain(),),
        }
    }
}

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }
    foreign_links {
        ConfigError(::std::num::ParseIntError);
        Io(::std::io::Error);
        SocketParse(::std::net::AddrParseError);
    }

    errors {
        ErrorStartingProcess(command: Command){
            description("Error while executing command")
            display("Error while executing command: '{:?}'", command)
        }
        ConfigurationError(t: String) {
            description("Configuration error")
            display("Configuration error: '{}'", t)
        }
    }
}
