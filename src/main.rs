use std::path::PathBuf;

use crate::helper::Helper::CLI;

mod helper;
mod render;
mod input;
mod filters;
mod processing;
mod models;
mod Dao;
fn main() {
    let mut clargs = CLI::new();
    clargs.Parse_Args();

    if clargs.dbg{
        println!("{clargs:?}");
    }






}
