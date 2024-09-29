//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore,Feeder};
use nix::unistd;
use std::ffi::CString;
use std::process;

use nix::unistd::ForkResult;
use std::env;             //追加
use std::path::Path;
use nix::errno::Errno;

pub struct Command {
    _text: String,
    args: Vec<String>,
    cargs: Vec<CString>,
}

impl Command {
    pub fn exec(&mut self, core: &mut ShellCore) {
        if self.args[0] == "exit" {
            eprintln!("exit");
            if self.args.len() > 1 {
                core.vars.insert("?".to_string(), self.args[1].clone());
            }

            let exit_status = match core.vars["?"].parse::<i32>() {
                Ok(n) => n % 256,
                Err(_) => {
                    println!("sush: exit: {}: numeric argument required", core.vars["?"]);
                    2
                },
            };

            process::exit(exit_status);
        }
        if self.args[0] == "cd" && self.args.len() > 1 {
            let path = Path::new(&self.args[1]);
            if env::set_current_dir(&path).is_err() {
                eprintln!("Cannot change directory");
            }
            return;
        }

        match unsafe{unistd::fork()} {
            Ok(ForkResult::Child) => {
                match unistd::execvp(&self.cargs[0], &self.cargs) {
                    Err(Errno::EACCES) => {
                        println!("sush: {}: Permission denied", &self.args[0]);
                        process::exit(126);
                    },
                    Err(Errno::ENOENT) => {
                        println!("{}: command not found", &self.args[0]);
                        process::exit(127);
                    },
                    Err(err) => {
                        println!("Failed to execute. {:?}", err);
                        process::exit(127);
                    },
                    _ => ()
                }
            },
            Ok(ForkResult::Parent { child } ) => {
                core.wait_process(child)
            },
            Err(err) => panic!("Failed to fork. {}", err),
        }
    }

    pub fn parse(feeder: &mut Feeder, _core: &mut ShellCore) -> Option<Command> {
        let line = feeder.consume(feeder.remaining.len());
        let args: Vec<String> = line
            .trim_end()
            .split(' ')
            .map(|w| w.to_string())
            .collect();

        let cargs: Vec<CString> = args
            .iter()
            .map(|w| CString::new(w.clone()).unwrap())
            .collect();

        if args.len() > 0 { // 1個以上の単語があればCommandのインスタンスを作成して返す
            Some( Command {_text: line, args, cargs } )
        }else{
            None // そうでなければ何も返さない
        }
    }
}
