#[macro_use]
extern crate stderr;

use clap::{Arg, Command};
use std::process;

fn main() {
    let m = Command::new(env!("CARGO_CRATE_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            Command::new("set")
                .arg(Arg::new("KEY").takes_value(true))
                .arg(Arg::new("VALUE").takes_value(true)),
        )
        .subcommand(Command::new("get").arg(Arg::new("VALUE").takes_value(true)))
        .subcommand(Command::new("rm").arg(Arg::new("KEY").takes_value(true)))
        .arg(Arg::new("V"))
        .get_matches();

    match m.subcommand() {
        Some(("set", _sub_set)) => {
            err!("unimplemented\n");
            process::exit(1);
        }
        Some(("get", _sub_get)) => {
            err!("unimplemented\n");
            process::exit(1);
        }
        Some(("rm", _)) => {
            err!("unimplemented\n");
            process::exit(1);
        }
        _ => {
            err!(env!("CARGO_PKG_VERSION"));
            process::exit(1);
        }
    }
}
