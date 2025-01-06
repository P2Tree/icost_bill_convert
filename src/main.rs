use clap::{self, Parser};
use std::error::Error;
use std::path::PathBuf;

mod arguments;
use arguments::Args;

mod zhifubao;
use zhifubao::handle_bill as zhifubao_handle;

mod weixin;
use weixin::handle_bill as weixin_handle;

mod output;
use output::OutputRecord;

type DynResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args = Args::parse();

    // get input bill path
    let zfb_bill = args.zhifubao_bill;
    let wx_bill = args.weixin_bill;
    if zfb_bill.is_none() && wx_bill.is_none() {
        println!("请提供至少一个账单文件");
        return Ok(());
    }

    // set output bill path
    let output_file = &args.output.unwrap_or(PathBuf::from("output.csv"));
    let user = &args.user;

    // save records
    let mut records: Vec<OutputRecord> = Vec::new();

    if let Some(zfb_bill) = zfb_bill {
        zhifubao_handle(user, &mut records, &zfb_bill);
    }

    if let Some(wx_bill) = wx_bill {
        weixin_handle(user, &mut records, &wx_bill);
    }

    assert!(!records.is_empty(), "没有读取到任何记录");

    // combine all bills
    OutputRecord::sort_by_time(&mut records);
    OutputRecord::check(&records);
    OutputRecord::write(output_file, &records).expect("write to new csv file error");

    // summary records
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
                println!(
                    "{} {}: 转账记录缺少目标账户，请手动添加",
                    record.date, record.source
                );
            }
        } else {
            println!(
                "{} {}: 未知的交易类型: {}，请手动处理",
                record.date, record.source, record.r#type
            );
        }
    }
    println!("汇总：");
    println!("支出记录数: {}", input_type_count);
    println!("收入记录数: {}", output_type_count);
    println!("转账记录数: {}", transfer_type_count);

    Ok(())
}
