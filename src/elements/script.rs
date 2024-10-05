//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::job::Job;
use crate::feeder::Feeder;
use crate::core::ShellCore;

pub struct Script {
    pub jobs: Vec<Job>,
    pub text: String,
}

impl Script {
    pub fn parse(text: &mut Feeder, core: &mut ShellCore) -> Option<Script> {
        if let Some(job) = Job::parse(text, core) {
            return Some( Script { text: job.text.clone(), jobs: vec!(job) } );
        }
        None
    }
}
