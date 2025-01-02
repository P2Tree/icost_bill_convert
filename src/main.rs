// 导入所需的库和模块
use csv::{ReaderBuilder, WriterBuilder};
use encoding_rs::GBK;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::Serialize;
use std::env;
use std::error::Error;

// 定义输出记录的结构体，使用 Serialize 特征以支持 CSV 序列化
#[derive(Serialize)]
struct OutputRecord {
    date: String,      // 日期
    r#type: String,    // 交易类型
    amount: String,    // 金额
    category1: String, // 分类1
    category2: String, // 分类2
    account1: String,  // 账户1
    account2: String,  // 账户2
    remark: String,    // 备注
    currency: String,  // 货币类型
    tag: String,       // 标签
}

// 读取输入文件并处理数据
fn read_input_file(input_file: &str) -> Result<Vec<OutputRecord>, Box<dyn Error>> {
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
        let amount = record.get(6).unwrap_or("").to_string();
        let payment_method = record.get(7).unwrap_or("").to_string();

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
fn write_output_file(output_file: &str, records: &[OutputRecord]) -> Result<(), Box<dyn Error>> {
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

// 主函数：处理命令行参数并协调整个程序的执行流程
fn main() -> Result<(), Box<dyn Error>> {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();

    // 检查参数数量是否正确
    if args.len() != 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(1);
    }

    // 设置输入和输出文件路径
    let input_file = &args[1];
    let output_file = input_file; // 输出文件与输入文件相同

    // 读取并处理输入文件
    let records = read_input_file(input_file)?;
    // 写入处理后的数据到输出文件
    write_output_file(output_file, &records)?;

    Ok(())
}
