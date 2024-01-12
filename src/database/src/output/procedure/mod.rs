use serde::Serialize;

use super::procedure_leg::ProcedureLeg;

pub mod arrival;
pub mod departure;

#[derive(Serialize)]
pub struct Transition {
    ident: String,
    legs: Vec<ProcedureLeg>,
}

pub fn mut_find_or_insert<T, P: FnMut(&T) -> bool>(vec: &mut Vec<T>, condition: P, val: T) -> &mut T {
    if let Some(index) = vec.iter().position(condition) {
        &mut vec[index]
    } else {
        vec.push(val);

        vec.last_mut().unwrap()
    }
}
