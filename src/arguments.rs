use clap::{self, Parser};
use std::error::Error;
use std::path::{Path, PathBuf};

use crate::DynResult;

#[derive(Parser, Debug)]
#[command(name = "icost-bill-convert", author, version, about)]
pub struct Args {
    #[clap(short='s', long="source", value_parser=parse_source)]
    pub source: Source,

    #[clap(short='i', long="input", value_parser=clap::value_parser!(PathBuf))]
    pub input: PathBuf,

    #[clap(short='o', long="output", value_parser=clap::value_parser!(PathBuf))]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    ZhiFuBao,
    WeiXin,
}

pub fn parse_source(source: &str) -> DynResult<Source> {
    match source {
        "zhifubao" => Ok(Source::ZhiFuBao),
        "weixin" => Ok(Source::WeiXin),
        _ => Err("Invalid source".into()),
    }
}
