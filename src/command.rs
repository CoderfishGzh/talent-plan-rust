use crate::errors::Result;
use crate::KvStore;
use clap::{Arg, ArgMatches, Command};

pub fn create_arg_matchs() -> ArgMatches {
    Command::new(env!("CARGO_CRATE_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            Command::new("set")
                .arg(Arg::new("KEY").takes_value(true))
                .arg(Arg::new("VALUE").takes_value(true)),
        )
        .subcommand(Command::new("get").arg(Arg::new("KEY").takes_value(true)))
        .subcommand(Command::new("rm").arg(Arg::new("KEY").takes_value(true)))
        .arg(Arg::new("V"))
        .get_matches()
}

pub fn set(mut kvs: KvStore, arg: &ArgMatches) -> Result<()> {
    let key = arg.value_of("KEY").unwrap();
    let value = arg.value_of("VALUE").unwrap();
    kvs.set(key.to_string(), value.to_string())
}

pub fn get(mut kvs: KvStore, arg: &ArgMatches) -> Result<()> {
    let key = arg.value_of("KEY").unwrap();
    return match kvs.get(key.to_string()) {
        Ok(v) => match v {
            None => {
                println!("Key not found");
                Ok(())
            }
            Some(value) => {
                println!("{}", value);
                Ok(())
            }
        },
        Err(e) => {
            println!("Key not found");
            Ok(())
        }
    };
}

pub fn rm(mut kvs: KvStore, arg: &ArgMatches) -> Result<()> {
    let key = arg.value_of("KEY").unwrap();
    kvs.remove(key.to_string())
}
