use csv::{ReaderBuilder, WriterBuilder};
use encoding_rs::{Encoding, GBK, UTF_8};
use encoding_rs_io::DecodeReaderBytesBuilder;
use log::{debug, error, info, warn};
use regex::Regex;
use std::path::{Path, PathBuf};

use crate::arguments::{self, User};
use crate::output::{check as output_check, sort_by_time, write_output_file as output_write};
use crate::{DynResult, OutputRecord};

// 读取输入文件并处理数据
pub fn read_input_file(input_file: &Path, user: &User) -> DynResult<Vec<OutputRecord>> {
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

        let source = "微信";
        let transaction_time = record.get(0).unwrap_or("").to_string();
        let transaction_type = record.get(1).unwrap_or("").to_string();
        let counterparty = record.get(2).unwrap_or("").to_string();
        let mut transaction_direction = record.get(4).unwrap_or("").to_string();
        let mut remark = record.get(3).unwrap_or("").to_string();
        let amount_str = record.get(5).unwrap_or("");

        let re = Regex::new(r"^\D*").unwrap(); // delete all non-digit characters until the first digit
        let amount = re
            .replace(amount_str, "")
            .to_string()
            .replace(",", "")
            .parse::<f32>()
            .map_err(|e| {
                format!(
                    "{} {}: 不支持的金额输入格式: {}",
                    transaction_time, source, e
                )
            })?;
        let mut account_from = record.get(6).unwrap_or("").to_string(); // 收入、支出账户和转账时的转出账户
        let mut account_to = String::from(""); // 只有在转账时使用，作为转入账户
        let status = record.get(7).unwrap_or("").to_string();
        let currency = match amount_str.chars().next() {
            Some('¥') => "CNY".to_string(),
            _ => "".to_string(),
        };

        // 跳过不关心的交易类型
        if status == "已全额退款" {
            debug!(
                "{} {}: 跳过全额退款交易: {:?}",
                transaction_time, source, record
            );
            continue;
        }

        // 处理特别的交易类型
        if transaction_direction == "/" && transaction_type.contains("转入零钱通") {
            transaction_direction = "转账".to_string();
            account_from = "零钱".to_string();
            account_to = "微信零钱通".to_string();
            remark = transaction_type.clone();
        }

        if status == "已存入零钱" && account_from == "/" {
            transaction_direction = "收入".to_string();
            account_from = "零钱".to_string();
        }

        account_from = append_user_postfix(&account_from, user);
        account_to = append_user_postfix(&account_to, user);

        // 处理特别的分类信息
        let (category1, category2) =
            filter_category(&counterparty, &remark, &transaction_direction);

        // 拼接备注信息
        let remark = remark + ": " + &counterparty;

        // 格式化日期
        let formatted_date = format_date(&transaction_time);

        let output_record = OutputRecord {
            date: formatted_date,
            r#type: transaction_direction,
            amount,
            category1,
            category2,
            account1: account_from,
            account2: account_to,
            remark,
            currency,
            tag: String::new(), // 暂时留空
            source: String::from(source),
        };

        records.push(output_record);
    }

    Ok(records)
}

// 格式化日期字符串
// 输入格式：year-month-day hour:minute:second
// 输出格式：year年month月day日 hour:minute:second
fn format_date(input: &str) -> String {
    let parts: Vec<&str> = input.split_whitespace().collect();
    assert!(parts.len() == 2, "日期时间格式不正确: {}", input,);

    let date_part = parts[0];
    let time_part = parts[1];

    let date_components: Vec<&str> = date_part.split('-').collect();
    assert!(date_components.len() == 3, "日期格式不正确: {}", date_part,);

    let year = date_components[0];
    let month = date_components[1].to_string();
    let day = date_components[2].to_string();

    let time_components: Vec<&str> = time_part.split(':').collect();
    assert!(time_components.len() == 3, "时间格式不正确: {}", time_part,);

    let hour = time_components[0].to_string();
    let minute = time_components[1].to_string();
    let second = time_components[2].to_string();

    format!(
        "{}年{}月{}日 {}:{}:{}",
        year, month, day, hour, minute, second
    )
}

fn filter_category(
    counterparty: &str,
    remark: &str,
    transaction_direction: &str,
) -> (String, String) {
    if transaction_direction == "转账" {
        return ("".to_string(), "".to_string());
    }

    let mut category1 = "未知".to_string();
    let mut category2 = "".to_string();
    if counterparty.contains("禹泉水处理设备") {
        category1 = "账单".to_string();
        category2 = "水费".to_string();
    } else if counterparty.contains("北京市顺义区妇幼保健院") {
        category1 = "医疗".to_string();
        category2 = "门诊".to_string();
    } else if counterparty.contains("易寄件") {
        category1 = "杂项".to_string();
        category2 = "快递费".to_string();
    } else if counterparty.contains("顺义鑫绿都生活超市后沙峪店") {
        category1 = "食材".to_string();
        category2 = "蔬菜".to_string();
    } else if counterparty.contains("永辉超市") {
        category1 = "食材".to_string();
        category2 = "蔬菜".to_string();
    }

    if remark.contains("霸王茶姬") {
        category1 = "餐饮".to_string();
        category2 = "饮料".to_string();
    }

    (category1, category2)
}

fn append_user_postfix(account: &str, user: &User) -> String {
    if !(account == "零钱" || account == "微信零钱通") {
        return account.to_string();
    }

    match user {
        User::Yang => account.to_string() + "-杨",
        User::Han => account.to_string() + "-韩",
    }
}

pub fn handle_bill(user: &arguments::User, records: &mut Vec<OutputRecord>, input_file: &PathBuf) {
    let input_file = Path::new(input_file);
    info!("处理账单文件: {}", input_file.display());
    println!("处理微信账单: {}", input_file.display());
    let current_records = read_input_file(input_file, user).expect("read input csv file error");
    println!("处理微信账单条目数量: {}", current_records.len());
    records.extend(current_records);
}
