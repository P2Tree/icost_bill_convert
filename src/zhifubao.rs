use csv::{ReaderBuilder, WriterBuilder};
use encoding_rs::GBK;
use encoding_rs_io::DecodeReaderBytesBuilder;
use log::{debug, error, warn};
use std::path::Path;

use crate::{DynResult, OutputRecord};

// 读取输入文件并处理数据
pub fn read_input_file(input_file: &Path) -> DynResult<Vec<OutputRecord>> {
    // 打开文件
    let file = std::fs::File::open(input_file)?;
    let decoder = DecodeReaderBytesBuilder::new()
        .encoding(Some(GBK))
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
        let mut transaction_type = record.get(5).unwrap_or("").to_string();
        let remark = record.get(4).unwrap_or("").to_string();
        let amount = record
            .get(6)
            .unwrap()
            .parse::<f32>()
            .expect("不支持的金额输入格式");
        let mut account_from = record.get(7).unwrap_or("").to_string();
        let mut account_to = String::from(""); // 只有在转账时使用，作为转入账户
        let status = record.get(8).unwrap_or("").to_string();

        // 处理特别的交易类型
        if status == "已关闭" {
            debug!("跳过已关闭交易：{:?}", record);
            continue;
        } else if status == "退款成功" {
            transaction_type = "退款".to_string();
        } else if status == "还款成功" && remark == "信用卡还款" {
            transaction_type = "转账".to_string();
            println!("需要手动添加还款目标卡，日期：{}", transaction_time);
            account_to = String::from("?????");
        }
        if account_from == "账户余额" {
            account_from = "支付宝零钱".to_string();
        }
        if transaction_type == "不计收支" {
            if remark.contains("余额宝") && remark.contains("收益发放") {
                transaction_type = "收入".to_string();
            }
            if remark.contains("余额宝-自动转入") {
                transaction_type = "转账".to_string();
                account_to = "余额宝".to_string();
                account_from = "支付宝零钱".to_string();
            }
        }

        // 格式化日期
        let formatted_date = format_date(&transaction_time);

        let output_record = OutputRecord {
            date: formatted_date,
            r#type: transaction_type,
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

    Ok(records)
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