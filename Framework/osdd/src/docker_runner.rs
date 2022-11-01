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

use crate::errors::ErrorKind::ErrorStartingProcess;
use crate::errors::*;
use crate::*;

/// Starts processes from the given commands
pub fn handle_processes(commands: Vec<CommandWithName>) -> Result<()> {
    //start al handlers
    for mut command_with_name in commands {
        command_with_name
            .command
            .spawn()
            .chain_err(|| ErrorStartingProcess(command_with_name.command))?;
    }
    Ok(())
}
