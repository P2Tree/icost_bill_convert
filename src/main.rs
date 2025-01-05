use clap::{self, Parser};
use log::{debug, info};
use serde::Serialize;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

mod arguments;
use arguments::{Args, Source};

mod zhifubao;
use zhifubao::handle_bill as zhifubao_handle;
use zhifubao::read_input_file as zhifubao_read;

mod weixin;
use weixin::handle_bill as weixin_handle;
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
    let zfb_bill = args.zhifubao_bill;
    let wx_bill = args.weixin_bill;
    if zfb_bill.is_none() && wx_bill.is_none() {
        println!("请提供至少一个账单文件");
        return Ok(());
    }

    let output_file = &args.output.unwrap_or(PathBuf::from("output.csv"));
    let user = &args.user;

    let mut records: Vec<OutputRecord> = Vec::new();

    if let Some(zfb_bill) = zfb_bill {
        zhifubao_handle(user, &mut records, &zfb_bill);
    }

    if let Some(wx_bill) = wx_bill {
        weixin_handle(user, &mut records, &wx_bill);
    }

    assert!(!records.is_empty(), "没有读取到任何记录");

    sort_by_time(&mut records);
    output_check(&records);
    output_write(output_file, &records).expect("write to new csv file error");

    let mut input_type_count = 0;
    let mut output_type_count = 0;
    let mut transfer_type_count = 0;
    for record in records {
        if record.r#type == "支出" {
            input_type_count += 1;
        } else if record.r#type == "收入" {
            output_type_count += 1;
        } else if record.r#type == "转账" {
            transfer_type_count += 1;
            if record.account2 == "未知" {
                println!("{}: 转账记录缺少目标账户，请手动添加", record.date);
            }
        } else {
            println!(
                "{}: 未知的交易类型: {}，请手动处理",
                record.date, record.r#type
            );
        }
    }
    println!("支出记录数: {}", input_type_count);
    println!("收入记录数: {}", output_type_count);
    println!("转账记录数: {}", transfer_type_count);

    Ok(())
}
