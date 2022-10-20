use clap::{arg, Arg, ArgMatches, Command};
use kvs::engine::*;
use kvs::errors::*;
use kvs::KvStore;
use std::net::IpAddr;
use std::{env, process};

fn main() {
    let arg_matches = Command::new("kvs-server")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            arg!(--addr <IpPort>)
                .required(false)
                .default_value("127.0.0.1:4000"),
        )
        .arg(
            arg!(--engine <ENGINENAME>)
                .required(true)
                .value_parser(["kvs", "sled"]),
        )
        .arg(Arg::new("V"))
        .get_matches();

    if let Err(e) = deal_command(arg_matches) {
        println!("Error: {:?}", e);
        process::exit(-1);
    }
}

fn deal_command(arg_matches: ArgMatches) -> Result<()> {
    let addr = arg_matches
        .get_one::<String>("addr")
        .unwrap()
        .parse::<IpAddr>()?;
    let engine_type = select_engine(arg_matches.get_one::<String>("engine").unwrap())?;

    // 根据 addr 和 engine 开启服务

    Ok(())
}

fn select_engine(engine: &String) -> Result<EngineType> {
    let dir = env::current_dir()?;

    // 判断engine是否合法
    return if engine.eq("kvs") {
        // 判断是否选择了错误的引擎
        if dir.join("sled").exists() {
            Err(KvsError::ErrorEngine)?;
        }
        Ok(EngineType::KvsStore)
    } else {
        // 判断是否选择了错误的引擎
        if dir.join("kvs").exists() {
            Err(KvsError::ErrorEngine)?;
        }
        Ok(EngineType::SledKvsEngine)
    };
}
