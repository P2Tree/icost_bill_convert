use clap::{self, Parser};
use std::path::PathBuf;

use crate::DynResult;

#[derive(Parser, Debug)]
#[command(name = "icost-bill-convert", author, version, about)]
pub struct Args {
    #[clap(short = 'z', long = "zfb-bill", value_parser=clap::value_parser!(PathBuf))]
    pub zhifubao_bill: Option<PathBuf>,

    #[clap(short = 'w', long = "wx-bill", value_parser=clap::value_parser!(PathBuf))]
    pub weixin_bill: Option<PathBuf>,

    #[clap(short='o', long="output", value_parser=clap::value_parser!(PathBuf))]
    pub output: Option<PathBuf>,

    #[clap(short='u', long="user", value_parser=parse_user)]
    pub user: User,
}

#[derive(Debug, Clone)]
pub enum User {
    Yang,
    Han,
}

pub fn parse_user(user: &str) -> DynResult<User> {
    match user {
        "yang" => Ok(User::Yang),
        "han" => Ok(User::Han),
        _ => Err("Invalid user".into()),
    }
}
