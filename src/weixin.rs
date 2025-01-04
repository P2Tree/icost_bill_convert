use csv::{ReaderBuilder, WriterBuilder};
use encoding_rs::{Encoding, GBK, UTF_8};
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::path::Path;
use log::{debug, warn, error};

use crate::{DynResult, OutputRecord};

// 读取输入文件并处理数据
pub fn read_input_file(input_file: &Path) -> DynResult<Vec<OutputRecord>> {
    // 打开文件
    let file = std::fs::File::open(input_file)?;
    let decoder = DecodeReaderBytesBuilder::new()
        .encoding(Some(UTF_8))
        .build(file);

    // 创建更灵活的 CSV 读取器配置
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true) // 允许不同的字段数
        .trim(csv::Trim::All) // 修剪所有字段的空白
        .from_reader(decoder);

    let mut records = Vec::new();
    let mut headers_found = false;

    for result in rdr.records() {
        let record = result?;

        if !headers_found {
            if record.get(0).map_or(false, |s| s.contains("交易时间")) {
                headers_found = true;
                continue; // 跳过标题行
            } else {
                continue; // 继续查找标题行
            }
        }

        let transaction_time = record.get(0).unwrap_or("").to_string();
        let transaction_type = record.get(1).unwrap_or("").to_string();
        let counterparty = record.get(2).unwrap_or("").to_string();
        let mut transaction_direction = record.get(4).unwrap_or("").to_string();
        let mut remark = record.get(3).unwrap_or("").to_string();
        let amount = record.get(5).unwrap_or("").parse::<u64>().expect("不支持的金额输入格式");
        let mut account_from = record.get(6).unwrap_or("").to_string();  // 收入、支出账户和转账时的转出账户
        let mut account_to = String::from("");  // 只有在转账时使用，作为转入账户
        let status = record.get(7).unwrap_or("").to_string();

        // 跳过不关心的交易类型
        // if transaction_type == "不计收支" {
        //     continue;
        // }

        // 处理特别的交易类型
        if transaction_direction == "/" {
            if transaction_type.contains("转入零钱通") {
                transaction_direction = "转账".to_string();
                account_from = "零钱".to_string();
                account_to = "零钱通".to_string();
                remark = transaction_type;
            }
        }

        if status == "已存入零钱" && account_from == "/" {
            transaction_direction = "收入".to_string();
            account_from = "零钱".to_string();
            remark = counterparty;
        }

        // 格式化日期
        let formatted_date = format_date(&transaction_time);

        let output_record = OutputRecord {
            date: formatted_date,
            r#type: transaction_direction,
            amount,
            category1: String::new(), // 暂时留空
            category2: String::new(), // 暂时留空
            account1: account_from,
            account2: account_to,
            remark,
            currency: "CNY".to_string(), // 默认值
            tag: String::new(),          // 暂时留空
        };

        records.push(output_record);
    }

    for r in records.iter() {
        debug!("{:?}", r);
    }
    Ok(records)
}

pub fn check(records: &[OutputRecord]) {
    let mut has_error = false;
    for record in records.iter() {
        // 检查“时间”
        // TODO: 

        // 检查“类型”
        let transaction_type = &record.r#type;
        if transaction_type != "支出" && transaction_type != "收入" && transaction_type != "转账" {
            warn!("未知的交易方向: {}", transaction_type);
            has_error = true;
            continue;
        }

        // 检查“金额”
        // TODO:

        // 检查“一级分类”
        // TODO:

        // 检查“二级分类”
        // TODO:

        // 检查“账户1”
        let account_from = &record.account1;
        if account_from == "" {
            warn!("账户1为空");
            has_error = true;
            continue;
        }

        // 检查“账户2”
        let account_to = &record.account2;
        if transaction_type == "转账" && account_to == "" {
            warn!("转账时账户2为空");
            has_error = true;
            continue;
        }

        // 检查“备注”
        // TODO:

        // 检查“货币”
        // TODO:

        // 检查“标签”
        // TODO:
    }

    if has_error {
        error!("检查失败，请检查程序逻辑");
        std::process::exit(1);
    }
}

// 将处理后的记录写入输出文件
pub fn write_output_file(output_file: &Path, records: &Vec<OutputRecord>) -> DynResult<()> {
    let mut wtr = WriterBuilder::new().from_path(output_file)?;

    for record in records {
        wtr.serialize(record)?;
    }

    wtr.flush()?;
    Ok(())
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
