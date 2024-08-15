//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use super::elem::Elem;
use super::{elem, error_msg, float, int, rev_polish, trenary, word};

pub fn pop_operand(stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<Elem, String> {
    let n = match stack.pop() {
        Some(Elem::Word(w, inc)) => {
            match word::to_operand(&w, 0, inc, core) {
                Ok(op) => op,
                Err(e) => return Err(e),
            }
        },
        Some(elem) => elem,
        None       => return Err("no operand".to_string()),
    };
    Ok(n)
}

fn bin_operation(op: &str, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
    match op {
    "=" | "*=" | "/=" | "%=" | "+=" | "-=" | "<<=" | ">>=" | "&=" | "^=" | "|=" 
          => word::substitution(op, stack, core),
        _ => bin_calc_operation(op, stack, core),
    }
}

fn bin_calc_operation(op: &str, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
    let right = match pop_operand(stack, core) {
        Ok(v)  => v,
        Err(e) => return Err(e),
    };
    let left = match pop_operand(stack, core) {
        Ok(v)  => v,
        Err(e) => return Err(e),
    };

    return match (left, right) {
        (Elem::Float(fl), Elem::Float(fr)) => float::bin_calc(op, fl, fr, stack),
        (Elem::Float(fl), Elem::Integer(nr)) => float::bin_calc(op, fl, nr as f64, stack),
        (Elem::Integer(nl), Elem::Float(fr)) => float::bin_calc(op, nl as f64, fr, stack),
        (Elem::Integer(nl), Elem::Integer(nr)) => int::bin_calc(op, nl, nr, stack),
        _ => panic!("SUSH INTERNAL ERROR: invalid operand"),
    };
}

fn unary_operation(op: &str, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
    let operand = match pop_operand(stack, core) {
        Ok(v)  => v,
        Err(e) => return Err(e),
    };

    match operand {
        Elem::Float(num) => match op {
            "+"  => stack.push( Elem::Float(num) ),
            "-"  => stack.push( Elem::Float(-num) ),
            _ => return Err("not supported operator for float number".to_string()),
        }
        Elem::Integer(num) => match op {
            "+"  => stack.push( Elem::Integer(num) ),
            "-"  => stack.push( Elem::Integer(-num) ),
            "!"  => stack.push( Elem::Integer(if num == 0 { 1 } else { 0 }) ),
            "~"  => stack.push( Elem::Integer( !num ) ),
            _ => panic!("SUSH INTERNAL ERROR: unknown unary operator"),
        }
        _ => panic!("SUSH INTERNAL ERROR: unknown operand"),
    }

    Ok(())
}

pub fn calculate(elements: &Vec<Elem>, core: &mut ShellCore) -> Result<Elem, String> {
    let mut comma_pos = vec![];
    for (i, e) in elements.iter().enumerate() {
        match e {
            Elem::BinaryOp(c) => {
                if c == "," {
                    comma_pos.push(i);
                }
            },
            _ => {},
        }
    }

    let mut left = 0;
    for i in 0..comma_pos.len() {
        let right = comma_pos[i];
        if let Err(e) = calculate_sub(&elements[left..right], core) {
            return Err(e);
        }
        left = right + 1;
    }

    calculate_sub(&elements[left..], core)
}

fn calculate_sub(elements: &[Elem], core: &mut ShellCore) -> Result<Elem, String> {
    if elements.len() == 0 {
        return Ok(Elem::Integer(0));
    }

    let rev_pol = match rev_polish::rearrange(elements) {
        Ok(ans) => ans,
        Err(e)  => return Err( error_msg::syntax(&elem::to_string(&e)) ),
    };

    let mut stack = vec![];

    for e in rev_pol {
        let result = match e {
            Elem::Integer(_) | Elem::Float(_) | Elem::Word(_, _) => {
                stack.push(e.clone());
                Ok(())
            },
            Elem::BinaryOp(ref op) => bin_operation(&op, &mut stack, core),
            Elem::UnaryOp(ref op)  => unary_operation(&op, &mut stack, core),
            Elem::Increment(n)     => inc(n, &mut stack, core),
            Elem::Ternary(left, right) => trenary::operation(&left, &right, &mut stack, core),
            _ => Err( error_msg::syntax(&elem::to_string(&e)) ),
        };

        if let Err(err_msg) = result {
            return Err(err_msg);
        }
    }

    if stack.len() != 1 {
        return Err( format!("unknown syntax error (stack inconsistency)",) );
    }

    match stack.pop() {
        Some(Elem::Integer(n)) => Ok(Elem::Integer(n)),
        Some(Elem::Float(f)) => Ok(Elem::Float(f)),
        Some(Elem::Word(w, inc)) => {
            match word::to_operand(&w, 0, inc, core) {
                Ok(Elem::Integer(n)) => Ok(Elem::Integer(n)),
                Ok(Elem::Float(f)) => Ok(Elem::Float(f)),
                Err(e) => Err(e),
                _      => Err("unknown word parse error".to_string()),
            }
        },
        _ => Err( format!("unknown syntax error",) ),
    }
}

fn inc(inc: i64, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
    match stack.pop() {
        Some(Elem::Word(w, inc_post)) => {
            match word::to_operand(&w, inc, inc_post, core) {
                Ok(op) => {
                    stack.push(op);
                    Ok(())
                },
                Err(e) => Err(e),
            }
        },
        _ => Err("invalid increment".to_string()),
    }
}
