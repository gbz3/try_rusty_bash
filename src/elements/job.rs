//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::ShellCore;
use crate::elements::script::Script;
use crate::feeder::Feeder;
use super::pipeline::Pipeline;

pub struct Job {
    pub pipelines: Vec<Pipeline>,
    pub text: String,
}

impl Job {
    pub fn parse(text: &mut Feeder, core: &mut ShellCore) -> Option<Job> {
        if let Some(pipeline) = Pipeline::parse(text, core) {
            return Some( Job { text: pipeline.text.clone(), pipelines: vec!(pipeline) } );
        }
        None
    }
}
