//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::ShellCore;
use crate::elem_command::Command;
use crate::elements::job::Job;
use crate::elements::script::Script;
use crate::feeder::Feeder;

pub struct Pipeline {
    pub commands: Vec<Command>,
    pub text: String,
}

impl Pipeline {
    pub fn parse(text: &mut Feeder, core: &mut ShellCore) -> Option<Pipeline> {
        if let Some(command) = Command::parse(text, core) {
            return Some( Pipeline { text: command.text.clone(), commands: vec!(command) } );
        }
        None
    }
}
