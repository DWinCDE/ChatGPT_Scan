use flexi_logger::{Age, Cleanup, Criterion, DeferredNow, Logger, Naming, Record, WriteMode};
use log::{Level, Record as LogRecord};
use chrono::Local;

pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    let date = Local::now().format("%Y-%m-%d").to_string();
    Logger::try_with_str("info")?
        .log_to_file(
            flexi_logger::FileSpec::default()
                .directory("logs")
                .basename(&format!("tri_arb_{}", date))
                .suffix("log")
        )
        .write_mode(WriteMode::BufferAndFlush)
        .rotate(
            Criterion::Size(10 * 1024 * 1024), // 10 MB file size
            Naming::Numbers,
            Cleanup::Never,  // Optional: clean up old logs after a certain period
        )
        .format(format_log)
        .start()?;
    Ok(())
}

fn format_log(writer: &mut dyn std::io::Write, now: &mut DeferredNow, record: &Record) -> std::io::Result<()> {
    write!(
        writer,
        "{} [{}] - {} - {}\n",
        now.format("%Y-%m-%d %H:%M:%S%.3f"),
        record.level(),
        record.module_path().unwrap_or("unknown"),
        record.args()
    )
}

pub fn log_with_tag(tag: &str, level: Level, msg: &str) {
    log::log!(target: tag, level, "{}", msg);
}
