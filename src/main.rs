use clap::{self, Parser};
use serde::Serialize;
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};

mod arguments;
use arguments::{Args, Source};

mod zhifubao;
use zhifubao::read_input_file as zhifubao_read;

mod weixin;
use weixin::read_input_file as weixin_read;

mod output;
use output::check as output_check;
use output::sort_by_time;
use output::write_output_file as output_write;

type DynResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

// 定义输出记录的结构体，使用 Serialize 特征以支持 CSV 序列化
#[derive(Serialize, Debug)]
struct OutputRecord {
    #[serde(rename = "日期")]
    date: String,
    #[serde(rename = "类型")]
    r#type: String,
    #[serde(rename = "金额")]
    amount: f32,
    #[serde(rename = "一级分类")]
    category1: String,
    #[serde(rename = "二级分类")]
    category2: String,
    #[serde(rename = "账户1")]
    account1: String,
    #[serde(rename = "账户2")]
    account2: String,
    #[serde(rename = "备注")]
    remark: String,
    #[serde(rename = "货币")]
    currency: String,
    #[serde(rename = "标签")]
    tag: String,
}

// 主函数：处理命令行参数并协调整个程序的执行流程
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // 获取命令行参数
    let args = Args::parse();

    // 设置输入和输出文件路径
    let input_files: Vec<&str> = args.input.split(',').collect();
    let output_file = &args.output.unwrap_or(PathBuf::from("output.csv"));

    let mut records: Vec<OutputRecord> = Vec::new();
    for input_file in input_files {
        let input_file = Path::new(input_file);
        match source_selector(input_file).unwrap() {
            Source::ZhiFuBao => {
                println!("处理支付宝账单: {}", input_file.display());
                let current_records = zhifubao_read(input_file).expect("read input csv file error");
                records.extend(current_records);
            }
            Source::WeiXin => {
                println!("处理微信账单: {}", input_file.display());
                let current_records = weixin_read(input_file).expect("read input csv file error");
                records.extend(current_records);
            }
        };
        sort_by_time(&mut records);
        output_check(&records);
        output_write(output_file, &records).expect("write to new csv file error");
    }

    Ok(())
}

fn source_selector(input_file: &Path) -> DynResult<Source> {
    let file_name = input_file.file_name().unwrap().to_str().unwrap();
    match file_name {
        "zhifubao.csv" => Ok(Source::ZhiFuBao),
        "weixin.csv" => Ok(Source::WeiXin),
        _ => Err("Invalid source".into()),
    }
}
