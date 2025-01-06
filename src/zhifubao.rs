use csv::{ReaderBuilder, WriterBuilder};
use encoding_rs::GBK;
use encoding_rs_io::DecodeReaderBytesBuilder;
use log::{debug, error, info, warn};
use std::path::{Path, PathBuf};

use crate::arguments::{self, User};
use crate::output::{check as output_check, sort_by_time, write_output_file as output_write};
use crate::{DynResult, OutputRecord};

// 读取输入文件并处理数据
pub fn read_input_file(input_file: &Path, user: &User) -> DynResult<Vec<OutputRecord>> {
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

        let source = "支付宝";
        let transaction_time = record.get(0).unwrap_or("").to_string();
        let counterparty = record.get(2).unwrap_or("").to_string();
        let mut transaction_type = record.get(5).unwrap_or("").to_string();
        let description = record.get(4).unwrap_or("").to_string();
        let amount = record.get(6).unwrap().parse::<f32>().map_err(|e| {
            format!(
                "{} {}: 不支持的金额输入格式: {}",
                transaction_time, source, e
            )
        })?;
        let mut account_from = record.get(7).unwrap_or("").to_string();
        let mut account_to = String::from(""); // 只有在转账时使用，作为转入账户
        let status = record.get(8).unwrap_or("").to_string();
        let remark = record.get(11).unwrap_or("").to_string();

        append_user_postfix(&account_from, user);
        append_user_postfix(&account_to, user);

        // 处理特别的交易类型
        if transaction_type == "不计收支" {
            if description.contains("余额宝") && description.contains("收益发放") {
                transaction_type = "收入".to_string();
            } else if description.contains("余额宝-自动转入") {
                transaction_type = "转账".to_string();
                account_to = "余额宝".to_string();
                account_from = "支付宝零钱".to_string();
            }
            if account_from.contains("亲情卡") {
                debug!(
                    "{} {}: 跳过亲情卡交易: {:?}",
                    transaction_time, source, record
                );
                continue;
            } else if account_from.contains("他人代付") {
                debug!(
                    "{} {}: 跳过他人代付交易: {:?}",
                    transaction_time, source, record
                );
                continue;
            }
            debug!(
                "{} {}: 跳过其他不计收支交易: {:?}",
                transaction_time, source, record
            );
            continue;
        }

        if status == "已关闭" || status == "交易关闭" {
            debug!(
                "{} {}: 跳过已关闭交易: {:?}",
                transaction_time, source, record
            );
            continue;
        } else if status == "退款成功" {
            transaction_type = "退款".to_string();
        } else if status == "还款成功" && description == "信用卡还款" {
            transaction_type = "转账".to_string();
            account_to = "未知".to_string();
            warn!(
                "{} {}: 需要手动添加还款目标卡: {:?}",
                transaction_time, source, record
            );
        }

        if amount == 0.0 {
            debug!(
                "{} {}: 跳过金额为0的交易: {:?}",
                transaction_time, source, record
            );
            continue;
        }

        account_from = append_user_postfix(&account_from, user);
        account_to = append_user_postfix(&account_to, user);

        account_from = account_from
            .split('&')
            .next()
            .unwrap_or(&account_from)
            .to_string();
        account_to = account_to
            .split('&')
            .next()
            .unwrap_or(&account_to)
            .to_string();

        // 处理特别的分类信息
        let (category1, category2) =
            filter_category(&counterparty, &description, &transaction_type, amount);

        // 拼接备注信息
        let remark = description + ": " + &remark;

        // 格式化日期
        let formatted_date = format_date(&transaction_time);

        let output_record = OutputRecord {
            date: formatted_date,
            r#type: transaction_type,
            amount,
            category1,
            category2,
            account1: account_from,
            account2: account_to,
            remark,
            currency: "CNY".to_string(), // 默认值
            tag: String::new(),          // 暂时留空
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
    assert!(parts.len() == 2, "日期时间格式不正确: {}", input);

    let date_part = parts[0];
    let time_part = parts[1];

    let date_components: Vec<&str> = date_part.split('-').collect();
    assert!(date_components.len() == 3, "日期格式不正确: {}", date_part);

    let year = date_components[0];
    let month = date_components[1].to_string();
    let day = date_components[2].to_string();

    let time_components: Vec<&str> = time_part.split(':').collect();
    assert!(time_components.len() == 3, "时间格式不正确: {}", time_part);

    let hour = time_components[0].to_string();
    let minute = time_components[1].to_string();
    let second = time_components[2].to_string();

    // 输出格式为 "year年month月day日 hour:minute:second"
    format!(
        "{}年{}月{}日 {}:{}:{}",
        year, month, day, hour, minute, second
    )
}

fn filter_category(
    counterparty: &str,
    remark: &str,
    transaction_type: &str,
    amount: f32,
) -> (String, String) {
    if transaction_type == "转账" {
        return ("".to_string(), "".to_string());
    }

    let mut category1 = "未知".to_string();
    let mut category2 = "".to_string();
    match counterparty {
        "北京一卡通" => {
            category1 = "交通".to_string();
            if amount < 2.0 {
                category2 = "公交".to_string();
            } else {
                category2 = "地铁".to_string();
            }
        }
        "饿了么" => {
            category1 = "餐饮".to_string();
            category2 = "外卖".to_string();
        }
        "兴全基金管理有限公司" => {
            if transaction_type == "收入" {
                category1 = "资本".to_string();
                category2 = "投资收入".to_string();
            } else if transaction_type == "支出" {
                category1 = "资本".to_string();
                category2 = "投资亏损".to_string();
            }
        }
        "中国移动" => {
            if remark.contains("话费充值") {
                category1 = "账单".to_string();
                category2 = "电话费".to_string();
            }
        }
        "蚂蚁森林" => {
            category1 = "意外收入".to_string();
        }
        "Steam" => {
            category1 = "网络".to_string();
            category2 = "游戏".to_string();
        }
        "众博康健大药房" => {
            category1 = "医疗".to_string();
            category2 = "药品".to_string();
        }
        "北京永辉超市有限公司" => {
            category1 = "食材".to_string();
            category2 = "蔬菜".to_string();
        }
        "北京大学口腔医院" => {
            category1 = "医疗".to_string();
            category2 = "牙齿".to_string();
        }
        "淮南牛肉汤" => {
            category1 = "餐饮".to_string();
            category2 = "三餐".to_string();
        }
        "汤鲜生浦项中心店" => {
            category1 = "餐饮".to_string();
            category2 = "三餐".to_string();
        }
        "滴滴出行（北京）网络平台技术有限公司" => {
            category1 = "交通".to_string();
            category2 = "打车".to_string();
        }
        _ => {}
    }

    match remark {
        "电费" => {
            category1 = "账单".to_string();
            category2 = "电费".to_string();
        }
        "火车票" => {
            category1 = "交通".to_string();
            category2 = "火车".to_string();
        }
        _ => {}
    }

    (category1, category2)
}

fn append_user_postfix(account: &str, user: &User) -> String {
    let mut account = account.to_string();
    if account == "账户余额" {
        account = String::from("支付宝零钱");
    }
    if !(account == "支付宝零钱" || account == "余额宝") {
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
    println!("处理支付宝账单: {}", input_file.display());
    let current_records = read_input_file(input_file, user).expect("read input csv file error");
    println!("处理支付宝账单条目数量: {}", current_records.len());
    records.extend(current_records);
}
