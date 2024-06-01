//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::Subword;

#[derive(Debug, Clone)]
pub struct SimpleSubword {
    pub text: String,
}

impl Subword for SimpleSubword {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}
}

impl SimpleSubword {
    fn new(s: &str) -> SimpleSubword {
        SimpleSubword {
            text: s.to_string(),
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<SimpleSubword> {
        let len = feeder.scanner_word(core);
        if len == 0 {
            None
        }else{
            Some(Self::new( &feeder.consume(len) ))
        }
    }
}
