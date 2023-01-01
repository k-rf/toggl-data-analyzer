use std::fs::File;
use std::{env, path::Path};

use polars::{export::num::integer::div_mod_floor, prelude::*};

fn main() -> Result<(), PolarsError> {
    let args: Vec<String> = env::args().collect();

    let input_path = Path::new(&args[1]);
    let output_path = format!(
        "{}/{}",
        "output_data",
        input_path.file_name().unwrap().to_str().unwrap()
    );

    let mut df = create_data_frame(LazyCsvReader::new(input_path).has_header(true).finish()?);

    let mut file = File::create(output_path)?;
    CsvWriter::new(&mut file)
        .has_header(true)
        .with_delimiter(b',')
        .finish(&mut df)?;

    Ok(())
}

/// データフレームを作成する
fn create_data_frame(frame: LazyFrame) -> DataFrame {
    frame
        .select([
            col("Client"),
            col("Duration").map(|duration| Ok(to_seconds(&duration)), GetOutput::default()),
        ])
        .groupby([col("Client")])
        .agg([
            col("Duration").sum(),
            col("Duration").alias("Seconds").sum(),
        ])
        .sort(
            "Duration",
            SortOptions {
                descending: (true),
                nulls_last: (true),
            },
        )
        .collect()
        .unwrap()
        .apply("Duration", to_hms)
        .unwrap()
        .clone()
}

/// \[h\]:mm:ss 形式の文字列を秒 (i32) に変換する
fn to_seconds(duration: &Series) -> Series {
    duration
        .utf8()
        .unwrap()
        .into_iter()
        .map(|duration| {
            let hms = duration.unwrap().split(":");

            hms.into_iter().enumerate().fold(0, |prev, (i, duration)| {
                let duration = duration.parse::<i32>().unwrap();

                prev + match i {
                    0 => duration * 60 * 60,
                    1 => duration * 60,
                    2 => duration,
                    _ => 0,
                }
            })
        })
        .collect()
}

/// 秒 (i32) を \[h\]:mm:ss 形式の文字列に変換する
fn to_hms(seconds: &Series) -> Series {
    seconds
        .i32()
        .unwrap()
        .into_iter()
        .map(|seconds| {
            let seconds = seconds.unwrap();

            let (m, s) = div_mod_floor(seconds, 60);
            let (h, m) = div_mod_floor(m, 60);

            format!("{}:{:0>2}:{:0>2}", h, m, s).to_string()
        })
        .collect()
}
