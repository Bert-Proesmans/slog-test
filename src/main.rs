#![feature(proc_macro, proc_macro_non_items, generators)]

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_envlogger;
extern crate slog_json;
extern crate slog_term;
extern crate tokio;
#[macro_use]
extern crate futures_await as futures;

use futures::prelude::*;
use slog::Drain;
use std::env;
use std::fs::OpenOptions;
use std::path::Path;

fn main() {
    setup();
}

fn get_logger() -> slog::Logger {
    let log_path = env::var("LOG_FILEPATH").unwrap_or_else(|_| "file.log".into());
    let log_path = Path::new(&log_path);

    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .expect("Failed to create log file");
    let file_logger = slog_json::Json::default(log_file);

    let console_decorator = slog_term::TermDecorator::new().build();
    let console_logger = slog_term::CompactFormat::new(console_decorator).build();

    let multiplexed_logger = slog::Duplicate::new(
        slog::LevelFilter::new(console_logger, slog::Level::Trace),
        file_logger,
    ).ignore_res();

    let logger = slog_async::Async::new(multiplexed_logger).build();
    slog::Logger::root(logger.fuse(), o!())
}

fn setup() {
    let root_logger = get_logger();
    let root_logger_two = root_logger.clone();

    let task = async_block!{
        let logger = root_logger.new(o!("id" => "1"));
        warn!(logger, "Within task 1");
        Ok::<(), String>(())
    };

    let task_logger = root_logger_two.new(o!("id" => "within task"));
    let task = task
        .and_then(move |_| future_work(task_logger))
        .and_then(|_| {
            println!("Finished");
            futures::lazy(|| futures::future::ok(()))
        })
        .map_err(|_| ());

    tokio::run(task);
}

#[async]
fn future_work(logger: slog::Logger) -> Result<(), String> {
    warn!(logger, "Within future_work");
    Ok(())
}
