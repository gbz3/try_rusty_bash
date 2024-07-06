//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub fn bg(core: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    for job in core.job_table.iter_mut() {
        job.send_cont();
    }
    0
}

pub fn jobs(core: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    for job in core.job_table.iter() {
        job.print();
    }
    0
}