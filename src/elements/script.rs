//SPDX-FileCopyrightText: 2022-2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use super::job::Job;
use crate::{error_message, Feeder, ShellCore};

enum Status{
    UnexpectedSymbol(String),
    NeedMoreLine,
    NormalEnd,
}

#[derive(Debug, Clone)]
pub struct Script {
    pub jobs: Vec<Job>,
    pub job_ends: Vec<String>,
    text: String,
}

impl Script {
    pub fn exec(&mut self, core: &mut ShellCore) {
        for (job, end) in self.jobs.iter_mut().zip(self.job_ends.iter()) {
            if core.word_eval_error {
                return;
            }
            job.exec(core, end == "&");
        }
    }

    pub fn get_text(&self) -> String { self.text.clone() }

    pub fn new() -> Script {
        Script {
            text: String::new(),
            jobs: vec![],
            job_ends: vec![],
        }
    }

    fn eat_job(feeder: &mut Feeder, core: &mut ShellCore, ans: &mut Script) -> bool {
        if let Some(job) = Job::parse(feeder, core){
            ans.text += &job.text.clone();
            ans.jobs.push(job);
            true
        }else{
            false
        }
    }

    fn eat_job_end(feeder: &mut Feeder, ans: &mut Script) -> bool {
        if feeder.starts_with(";;") || feeder.starts_with(";&") {
            ans.job_ends.push("".to_string());
            return true;
        }
        let len = feeder.scanner_job_end();
        let end = &feeder.consume(len);
        ans.job_ends.push(end.clone());
        ans.text += &end;
        len != 0
    }

    fn check_nest(&self, feeder: &mut Feeder) -> Status {
        let nest = feeder.nest.last()
                   .expect(&error_message::internal_str("empty nest"));

        match ( nest.1.iter().find(|e| feeder.starts_with(e)), self.jobs.len() ) {
            ( Some(end), 0 ) => return Status::UnexpectedSymbol(end.to_string()),
            ( Some(_), _)    => return Status::NormalEnd,
            ( None, _)       => {}, 
        }

        if feeder.len() > 0 {
            let remaining = feeder.consume(feeder.len());
            let first_token = remaining.split(" ").nth(0).unwrap().to_string();
            return Status::UnexpectedSymbol(first_token);
        }

        match nest.1.len() {
            0 => Status::NormalEnd,
            _ => Status::NeedMoreLine,
        }
    }

    fn unalias(&mut self, core: &mut ShellCore) {
        for a in core.data.alias_memo.iter().rev() {
            self.text = self.text.replace(&a.1, &a.0);
        }

        core.data.alias_memo.clear();
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore,
                 permit_empty: bool) -> Option<Script> {
        let mut ans = Self::new();
        
        if permit_empty {
            ans.jobs.push(Job::new());
            ans.job_ends.push("".to_string());
        }

        loop {
            while Self::eat_job(feeder, core, &mut ans) 
               && Self::eat_job_end(feeder, &mut ans) {}

            match ans.check_nest(feeder){
                Status::NormalEnd => {
                    ans.unalias(core);
                    return Some(ans)
                },
                Status::UnexpectedSymbol(s) => {
                    eprintln!("Unexpected token: {}", s);
                    core.data.set_param("?", "2");
                    break;
                },
                Status::NeedMoreLine => {
                    if ! feeder.feed_additional_line(core) {
                        break;
                    }
                },
            }
        }

        feeder.consume(feeder.len());
        core.data.alias_memo.clear();
        return None;
    }
}
