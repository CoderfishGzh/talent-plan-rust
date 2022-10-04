#[macro_use]
extern crate stderr;
// use kvs::command::{create_arg_matchs, get, rm, set};
use kvs::command::{create_arg_matchs, get, rm, set};
use kvs::errors::Result;
use kvs::KvStore;
use std::env::current_dir;
use std::process;

fn main() -> Result<()> {
    let m = create_arg_matchs();
    let kvs = KvStore::open(current_dir()?)?;
    match m.subcommand() {
        Some(("set", sub_set)) => {
            if let Err(e) = set(kvs, sub_set) {
                err!("{:?}", e);
                process::exit(1);
            }
            process::exit(0);
        }
        Some(("get", arg)) => {
            if let Err(e) = get(kvs, arg) {
                panic!("{:?}", e);
            }
        }
        Some(("rm", arg)) => {
            if let Err(_) = rm(kvs, arg) {
                print!("Key not found");
                process::exit(1);
            }
            process::exit(0);
        }
        _ => {
            err!(env!("CARGO_PKG_VERSION"));
            process::exit(1);
        }
    }

    // test()?;
    Ok(())
}
