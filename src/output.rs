use crate::{DynResult, OutputRecord};
use csv::WriterBuilder;
use log::{error, warn};
use std::{path::Path, process::Output};

pub fn sort_by_time(records: &mut Vec<OutputRecord>) {
    records.sort_by(|a, b| b.date.cmp(&a.date));
}

pub fn check(records: &[OutputRecord]) {
    let mut has_error = false;
    for record in records.iter() {
        // 检查“时间”
        // TODO:

        // 检查“类型”
        let transaction_type = &record.r#type;
        if transaction_type != "支出"
            && transaction_type != "收入"
            && transaction_type != "转账"
            && transaction_type != "退款"
        {
            warn!(
                "未知的交易方向: {}, 日期: {}",
                transaction_type, record.date
            );
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
        if account_from.is_empty() {
            warn!("账户1为空");
            has_error = true;
            continue;
        }

        // 检查“账户2”
        let account_to = &record.account2;
        if transaction_type == "转账" && account_to.is_empty() {
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
    println!("写入输出文件: {}", output_file.display());
    let mut wtr = WriterBuilder::new().from_path(output_file)?;

    for record in records {
        wtr.serialize(record)?;
    }

    wtr.flush()?;
    Ok(())
}
