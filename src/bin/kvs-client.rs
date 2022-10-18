use clap::{arg, Arg, ArgMatches, Command};
use kvs::errors::{KvsError, Result};
use std::net::SocketAddr;
use std::process;
use stderr::err;

fn main() {
    let arg_matches = Command::new("kvs-client")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            Command::new("set")
                .about("set the k-v")
                .arg(Arg::new("KEY").takes_value(true).required(true))
                .arg(Arg::new("VALUE").takes_value(true).required(true))
                .arg(
                    arg!(--addr <IpPort>)
                        .required(false)
                        .default_value("127.0.0.1:4000"),
                ),
        )
        .subcommand(
            Command::new("get")
                .arg(Arg::new("KEY").takes_value(true))
                .arg(
                    arg!(--addr <IpPort>)
                        .required(false)
                        .default_value("127.0.0.1:4000"),
                ),
        )
        .subcommand(
            Command::new("rm")
                .arg(Arg::new("KEY").takes_value(true))
                .arg(
                    arg!(--addr <IpPort>)
                        .required(false)
                        .default_value("127.0.0.1:4000"),
                ),
        )
        .arg(Arg::new("V"))
        .get_matches();

    if let Err(error) = deal_command(arg_matches) {
        err!("error: {:?}", error);
        process::exit(1);
    }
}

fn deal_command(arg_matches: ArgMatches) -> Result<()> {
    match arg_matches.subcommand() {
        Some(("set", command)) => {
            // get the arg
            let _key = command.value_of("KEY").unwrap();
            let _value = command.value_of("VALUE").unwrap();
            let _addr: SocketAddr = command.value_of("IpPort").unwrap().parse()?;
        }
        Some(("get", command)) => {
            // get the arg
            let _key = command.value_of("KEY").unwrap();
            let _addr: SocketAddr = command.value_of("IpPort").unwrap().parse()?;
        }
        Some(("rm", command)) => {
            // get the arg
            let _key = command.value_of("KEY").unwrap();
            let _addr: SocketAddr = command.value_of("IpPort").unwrap().parse()?;
        }
        _ => {
            return Err(KvsError::UnKnownCommandError);
        }
    }

    Ok(())
}
