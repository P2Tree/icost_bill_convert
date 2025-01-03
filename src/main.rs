use clap::{self, Parser};
use serde::Serialize;
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};

mod arguments;
use arguments::{Args, Source};

mod zhifubao;
use zhifubao::read_input_file as zhifubao_read;
use zhifubao::write_output_file as zhifubao_write;

mod weixin;
use weixin::read_input_file as weixin_read;
use weixin::write_output_file as weixin_write;

type DynResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

// 定义输出记录的结构体，使用 Serialize 特征以支持 CSV 序列化
#[derive(Serialize, Debug)]
struct OutputRecord {
    #[serde(rename = "日期")]
    date: String,
    #[serde(rename = "类型")]
    r#type: String,
    #[serde(rename = "金额")]
    amount: String,
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

// 格式化日期字符串
// 输入格式：year/month/day hour:minute
// 输出格式：year 年 month 月 day 日 hour:minute:second
fn format_date(input: &str) -> String {
    // 输入格式为 "year/month/day hour:minute"
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() != 2 {
        return input.to_string(); // 返回原始字符串以防格式不正确
    }

    let date_part = parts[0];
    let time_part = parts[1];

    let date_components: Vec<&str> = date_part.split('/').collect();
    if date_components.len() != 3 {
        return input.to_string(); // 返回原始字符串以防格式不正确
    }

    let year = date_components[0];
    let month = date_components[1];
    let day = date_components[2];

    // 输出格式为 "year 年 month 月 day 日 hour:minute:second"
    format!("{} 年 {} 月 {} 日 {}", year, month, day, time_part)
}

// 主函数：处理命令行参数并协调整个程序的执行流程
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // 获取命令行参数
    let args = Args::parse();

    // 设置输入和输出文件路径
    let input_file = &args.input;
    let output_file = match &args.output {
        Some(output_file) => output_file,
        None => &args.input,
    };

    match args.source {
        Source::ZhiFuBao => {
            let records = zhifubao_read(input_file).expect("read input csv file error");
            zhifubao_write(output_file, &records).expect("write to new csv file error");
        }
        Source::WeiXin => {
            let records = weixin_read(input_file).expect("read input csv file error");
            weixin_write(output_file, &records).expect("write to new csv file error");
        }
    }

    Ok(())
}
