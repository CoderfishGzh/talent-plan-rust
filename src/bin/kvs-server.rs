use clap::{arg, Arg, Command};

fn main() {
    let arg_matches = Command::new("kvs-server")
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
        .arg(
            arg!(--addr <IpPort>)
                .required(false)
                .default_value("127.0.0.1:4000"),
        )
        .arg(Arg::new("V"))
        .get_matches();
}
