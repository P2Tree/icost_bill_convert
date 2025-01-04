use csv::{ReaderBuilder, WriterBuilder};
use encoding_rs::GBK;
use encoding_rs_io::DecodeReaderBytesBuilder;
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
        let transaction_type = record.get(1).unwrap_or("").to_string();
        let remark = record.get(4).unwrap_or("").to_string();
        let amount = record.get(6).unwrap_or("").parse::<u64>().expect("不支持的金额输入格式");
        let payment_method = record.get(7).unwrap_or("").to_string();

        // 跳过不关心的交易类型
        if transaction_type == "不计收支" {
            continue;
        }

        // 格式化日期
        let formatted_date = format_date(&transaction_time);

        let output_record = OutputRecord {
            date: formatted_date,
            r#type: transaction_type,
            amount,
            category1: String::new(), // 暂时留空
            category2: String::new(), // 暂时留空
            account1: payment_method,
            account2: String::new(), // 暂时留空
            remark,
            currency: "CNY".to_string(), // 默认值
            tag: String::new(),          // 暂时留空
        };

        records.push(output_record);
    }

    Ok(records)
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
