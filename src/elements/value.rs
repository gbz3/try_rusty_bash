//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::combine;
use crate::debuginfo::DebugInfo;
use crate::Feeder;
use crate::abst_elems::*;
use crate::abst_elems::ArgElem;
use crate::elements::subarg_tilde::SubArgTildePrefix;
use crate::abst_elems::CommandElem;

pub struct Value {
    pub text: String,
    pub pos: DebugInfo,
    pub subvalues: Vec<Box<dyn ArgElem>>,
}

impl Value {
    pub fn new() -> Value {
        Value {
            text: "".to_string(),
            pos: DebugInfo{lineno: 0, pos: 0, comment: "".to_string()},
            subvalues: vec![],
        }
    }

    // single quoted arg or double quoted arg or non quoted arg 
    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Value> {
        if text.len() == 0 {
            return None;
        }

        let mut ans = Value{
            text: "".to_string(),
            pos: DebugInfo::init(text),
            subvalues: vec![],
        };

        if let Some(result) = SubArgTildePrefix::parse(text) {
            ans.text += &result.get_text();
            ans.subvalues.push(Box::new(result));
        }
    
        while let Some(result) = subvalue(text, conf) {
            ans.text += &(*result).get_text();
            ans.subvalues.push(result);
    
            if text.len() == 0 {
                break;
            };
        };
    
        if ans.text.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}

impl CommandElem for Value {
    fn parse_info(&self) -> Vec<String> {
        let mut ans = vec!(format!("    arg      : '{}' ({})",
                              self.text.clone(), self.pos.get_text()));
        for sub in &self.subvalues {
            ans.push("        subvalue      : ".to_owned() + &*sub.get_text());
        };

        ans
    }

    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        let mut subevals = vec![];
        for sa in &mut self.subvalues {
            let vs = sa.eval(conf);
            subevals.push(vs);
        }

        let mut strings = vec![];

        for ss in subevals {
            strings = combine(&mut strings, ss);
        }

        let mut ans = vec![];
        for v in strings {
            ans.append(&mut v.clone());
        }
        ans
    }

    fn get_text(&self) -> String { self.text.clone() }
}

