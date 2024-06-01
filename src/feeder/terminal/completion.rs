//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::core::builtins::completion;
use crate::elements::command::simple::SimpleCommand;
use crate::elements::command::Command;
use crate::elements::io::pipe::Pipe;
use crate::feeder::terminal::Terminal;
use std::path::Path;
use termion::cursor::DetectCursorPos;
use unicode_width::UnicodeWidthStr;

fn str_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

fn common_length(chars: &Vec<char>, s: &String) -> usize {
    let max_len = chars.len();
    for (i, c) in s.chars().enumerate() {
        if i >= max_len || chars[i] != c {
            return i;
        }
    }
    max_len
}

fn common_string(paths: &Vec<String>) -> String {
    if paths.len() == 0 {
        return "".to_string();
    }

    let ref_chars: Vec<char> = paths[0].chars().collect();
    let mut common_len = ref_chars.len();

    for path in &paths[1..] {
        let len = common_length(&ref_chars, &path);
        common_len = std::cmp::min(common_len, len);
    }

    ref_chars[..common_len].iter().collect()
}

fn is_dir(s: &str, core: &mut ShellCore) -> bool {
    let tilde_prefix = "~/".to_string();
    let tilde_path = core.data.get_param("HOME").to_string() + "/";

    Path::new(&s.replace(&tilde_prefix, &tilde_path)).is_dir()
}

impl Terminal {
    pub fn completion(&mut self, core: &mut ShellCore, tab_num: usize) {
        self.set_completion_info(core);

        if ! Self::set_custom_compreply(core)
        && ! self.set_default_compreply(core) {
            self.cloop();
            return;
        }

        match tab_num  {
            1 => self.try_completion(core),
            _ => self.show_list(&core.data.get_array_all("COMPREPLY"), tab_num),
        }
    }

    fn set_custom_compreply(core: &mut ShellCore) -> bool {
        let cur_pos = Self::get_cur_pos(core);
        let prev_pos = cur_pos - 1;
        let word_num = core.data.get_array_len("COMP_WORDS") as i32;

        if prev_pos < 0 || prev_pos >= word_num {
            return false;
        }

        let prev_word = core.data.get_array("COMP_WORDS", &prev_pos.to_string());
        let cur_word = core.data.get_array("COMP_WORDS", &cur_pos.to_string());

        match core.completion_functions.get(&prev_word) {
            Some(value) => {
                let command = format!("cur={} {}", &cur_word, &value); //TODO: cur should be set
                let mut feeder = Feeder::new(&command);                // by bash-completion

                if let Some(mut a) = SimpleCommand::parse(&mut feeder, core) {
                    let mut dummy = Pipe::new("".to_string());
                    a.exec(core, &mut dummy);
                }
                true
            },
            _ => false
        }
    }

    fn get_cur_pos(core: &mut ShellCore) -> i32 {
        match core.data.get_param("COMP_CWORD").parse::<i32>() {
            Ok(i) => i,
            _     => panic!("SUSH INTERNAL ERROR: no COMP_CWORD"),
        }
    }

    pub fn set_default_compreply(&mut self, core: &mut ShellCore) -> bool {
        let pos = core.data.get_param("COMP_CWORD").to_string();
        let last = core.data.get_array("COMP_WORDS", &pos);

        let (tilde_prefix, tilde_path, last_tilde_expanded) = Self::set_tilde_transform(&last, core);

        let mut args = vec!["".to_string(), "".to_string(), last_tilde_expanded.to_string()];
        let list = match pos == "0" {
            true  => completion::compgen_c(core, &mut args),
            false => completion::compgen_f(core, &mut args),
        };

        if list.len() == 0 {
            return false;
        }

        let tmp = list.iter().map(|p| p.replacen(&tilde_path, &tilde_prefix, 1)).collect();
        core.data.set_array("COMPREPLY", &tmp);
        true
    }

    pub fn try_completion(&mut self, core: &mut ShellCore) {
        let pos = core.data.get_param("COMP_CWORD").to_string();
        let target = core.data.get_array("COMP_WORDS", &pos);

        //if core.data.arrays[0]["COMPREPLY"].len() == 1 {
        if core.data.get_array_len("COMPREPLY") == 1 {
            //let output = core.data.arrays[0]["COMPREPLY"][0].clone();
            let output = core.data.get_array("COMPREPLY", "0");
            let tail = match is_dir(&output, core) {
                true  => "/",
                false => " ",
            };
            self.replace_input(&(output + tail));
            return;
        }

//        let common = common_string(&core.data.parameters[0]["COMPREPLY"]);
        let common = common_string(&core.data.get_array_all("COMPREPLY"));
        if common.len() != target.len() {
            self.replace_input(&common);
            return;
        }
        self.cloop();
    }

    fn normalize_tab(&mut self, row_num: usize, col_num: usize) {
        let i = (self.tab_col*row_num as i32 + self.tab_row)%((row_num*col_num) as i32);
        self.tab_col = i/(row_num as i32);
        self.tab_row = i%(row_num as i32);
    }

    fn show_list(&mut self, list: &Vec<String>, tab_num: usize) {
        let widths: Vec<usize> = list.iter().map(|s| str_width(s)).collect();
        let max_entry_width = widths.iter().max().unwrap_or(&1000) + 1;
        let col_num = std::cmp::min(
            std::cmp::max(Terminal::size().0 / max_entry_width, 1),
            list.len());
        let row_num = (list.len()-1) / col_num + 1;
        self.completion_candidate = String::new();

        if tab_num > 2 {
            self.normalize_tab(row_num, col_num);
        }

        eprintln!("\r");
        for row in 0..row_num {
            for col in 0..col_num {
                let tab = self.tab_row == row as i32 && self.tab_col == col as i32;
                self.print_an_entry(list, &widths, row, col, 
                    row_num, max_entry_width, tab);
            }
            print!("\r\n");
        }

        let terminal_row_num = Terminal::size().1;
        let (cur_col, cur_row) = self.stdout.cursor_pos().unwrap();

        self.check_scroll();
        match cur_row as usize == terminal_row_num {
            true => {
                let back_row = std::cmp::max(cur_row as i16 - row_num as i16, 1);
                self.write(&termion::cursor::Goto(cur_col, back_row as u16).to_string());
                print!("\x1b[1A");
                self.flush();
            },
            false => self.rewrite(false),
        }
    }

    fn print_an_entry(&mut self, list: &Vec<String>, widths: &Vec<usize>,
        row: usize, col: usize, row_num: usize, width: usize, nega: bool) {
        let i = col*row_num + row;
        if i >= list.len() {
            return;
        }

        let space_num = width - widths[i];
        let s = String::from_utf8(vec![b' '; space_num]).unwrap();
        if nega {
            print!("\x1b[01;7m{}{}\x1b[00m", list[i], &s);
            self.completion_candidate = list[i].clone();
        }else{
            print!("{}{}", list[i], &s);
        }
    }

    pub fn replace_input(&mut self, to: &String) {
        while self.head > self.prompt.chars().count() 
        && self.head > 0 && self.chars[self.head-1] != ' ' {
            self.backspace();
        }
        while self.head < self.chars.len() 
        && self.chars[self.head] != ' ' {
            self.delete();
        }

        let to_escaped = if to.ends_with(" ") {
            let mut tmp = to.to_string();
            tmp.pop();
            tmp.replace(" ", "\\ ") + " "
        }else {
            to.replace(" ", "\\ ").to_string()
        };

        for c in to_escaped.chars() {
            self.insert(c);
            self.check_scroll();
        }

        if to.ends_with(" ") 
        && self.head < self.chars.len() 
        && self.chars[self.head] == ' ' {
            self.backspace();
        }
    }

    fn set_tilde_transform(last: &str, core: &mut ShellCore) -> (String, String, String) {
        let tilde_prefix;
        let tilde_path;
        let last_tilde_expanded;

        if last.starts_with("~/") {
            tilde_prefix = "~/".to_string();
            tilde_path = core.data.get_param("HOME").to_string() + "/";
            last_tilde_expanded = last.replacen(&tilde_prefix, &tilde_path, 1);
        }else{
            tilde_prefix = String::new();
            tilde_path = String::new();
            last_tilde_expanded = last.to_string();
        }

        (tilde_prefix, tilde_path, last_tilde_expanded)
    }

    fn set_completion_info(&mut self, core: &mut ShellCore){
        let pcc = self.prompt.chars().count();
        let s = self.get_string(pcc);
        let mut ws = s.split(" ").map(|e| e.to_string()).collect::<Vec<String>>();
        ws.retain(|e| e != "");
        core.data.set_array("COMP_WORDS", &ws);

        let s: String = self.chars[pcc..self.head].iter().collect();
        let mut ws = s.split(" ").map(|e| e.to_string()).collect::<Vec<String>>();
        ws.retain(|e| e != "");
        let mut num = ws.len();

        match s.chars().last() {
            Some(' ') => {},
            Some(_) => num -= 1,
            _ => {},
        }
        core.data.set_param("COMP_CWORD", &num.to_string());
    }
}
