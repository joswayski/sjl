use crate::{
    log_event::LogEvent,
    log_level::LogLevel,
    logger_options::{LOGGER_INITIALIZED, LoggerOptions},
    timestamp::FormattedTimestamp,
};
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use crossbeam_queue::ArrayQueue;
use serde::Serialize;
use serde_json::{Map, Value};
use std::{
    io::Write,
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

const OVERSIZED_LOG_PREVIEW_LENGTH: usize = 200; // todo allow override?
const OVERSIZED_LOG_RESET_WINDOW: Duration = Duration::from_hours(4); // todo allow override?

#[must_use = "Logger does nothing unless you keep it and call log methods like `.info()`"]
pub struct Logger {
    pub(crate) sender: Option<Sender<Vec<u8>>>,
    pub(crate) worker: Option<std::thread::JoinHandle<()>>,
    pub(crate) buffer_pool: Arc<ArrayQueue<Vec<u8>>>,
    pub(crate) buffer_pool_initial_capacity: usize,

    // Options
    pub(crate) min_level: LogLevel,
    pub(crate) timestamp_format: Option<&'static str>,
    pub(crate) context: Map<String, Value>,
    pub(crate) pretty: bool,
}

impl Default for Logger {
    fn default() -> Self {
        LoggerOptions::default().init()
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        // Drop the sender so worker gets Disconnected
        self.sender.take();

        // Wait for thread to flush and exit
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }

        // Let other creations happen once dropped
        let _ = LOGGER_INITIALIZED.swap(false, Ordering::SeqCst);
    }
}

impl Logger {
    pub fn builder() -> LoggerOptions {
        LoggerOptions::default()
    }
    pub fn new() -> Self {
        LoggerOptions::default().init()
    }
    pub fn info<CustomData: Serialize>(&self, message: impl AsRef<str>, custom_data: CustomData) {
        self.log(LogLevel::Info, message.as_ref(), custom_data);
    }
    pub fn warn<CustomData: Serialize>(&self, message: impl AsRef<str>, custom_data: CustomData) {
        self.log(LogLevel::Warn, message.as_ref(), custom_data);
    }
    pub fn error<CustomData: Serialize>(&self, message: impl AsRef<str>, custom_data: CustomData) {
        self.log(LogLevel::Error, message.as_ref(), custom_data);
    }
    pub fn debug<CustomData: Serialize>(&self, message: impl AsRef<str>, custom_data: CustomData) {
        self.log(LogLevel::Debug, message.as_ref(), custom_data);
    }

    fn log<CustomData: Serialize>(
        &self,
        log_level: LogLevel,
        message: impl AsRef<str>,
        custom_data: CustomData,
    ) {
        if log_level.severity() < self.min_level.severity() {
            return;
        }

        if self.sender.is_none() {
            return;
        }

        // Don't serialize the empty data: () in the log event to null, just skip it
        let data = if size_of::<CustomData>() == 0 {
            None
        } else {
            Some(&custom_data)
        };

        let log_event = LogEvent {
            context: &self.context,
            level: log_level.as_str(),
            // TODO custom timestamp key with custom serialize impl
            timestamp: FormattedTimestamp::new(self.timestamp_format),
            data,
            message: message.as_ref(),
        };

        // get a buffer from the pool instead of creating one each time
        let mut buf = self
            .buffer_pool
            .pop()
            .unwrap_or_else(|| Vec::with_capacity(self.buffer_pool_initial_capacity));
        buf.clear(); // just in case

        let result = if self.pretty {
            serde_json::to_writer_pretty(&mut buf, &log_event)
        } else {
            serde_json::to_writer(&mut buf, &log_event)
        };

        if let Err(e) = result {
            eprintln!("Error ocurred converting log event to bytes. Error: {e}");
            // Extra check, re-clear the buffer before putting it back
            buf.clear();

            // Return the buffer to the pool if we errored
            let _ = self.buffer_pool.push(buf);
            return;
        };

        // newline between logs
        buf.push(b'\n');

        if let Some(sender) = &self.sender {
            let _ = sender.send(buf);
        }
    }

    pub(crate) fn handle_messages(
        worker: Receiver<Vec<u8>>,
        buffer_pool: Arc<ArrayQueue<Vec<u8>>>,
        buffer_pool_max_capacity: usize,
        buffer_pool_initial_capacity: usize,
        flush_at_bytes: usize,
        flush_at_messages: u16,
        flush_interval: Duration,
    ) -> std::thread::JoinHandle<()> {
        // Spawn a dedicated thread for logs
        std::thread::spawn(move || {
            let mut batch = Vec::<u8>::with_capacity(flush_at_bytes);
            let mut batch_message_count: u16 = 0;
            let mut oversized_messages_count: usize = 0;
            let mut oversized_messages_window = Instant::now();
            let mut total_messages_count: usize = 0;

            loop {
                match worker.recv_timeout(flush_interval) {
                    Ok(mut log_buffer) => {
                        batch.extend_from_slice(&log_buffer);
                        batch_message_count += 1; // this resets per batch

                        // Reset the window if its expired
                        if oversized_messages_window.elapsed() > OVERSIZED_LOG_RESET_WINDOW {
                            total_messages_count = 0;
                            oversized_messages_count = 0;
                            oversized_messages_window = Instant::now();
                        }

                        // this is global so that we can give a % of oversized logs
                        total_messages_count += 1;

                        // Check if the log that just came in made the vec grow
                        // past a certain size and trim it down if it did.
                        // This has to come after clear because shrink_to docs:
                        // `The capacity will remain at least as large as both the length and the supplied value`
                        // So if we shrink first with items still in it, it'll still be the size of the items inside
                        // even though the capacity provided is smaller: max(len(), MAX_BUFFER_POOL_VECTOR_SIZE)
                        // We could also drop the buffer here when it happens, the buffer pool size would shrink
                        // by 1 and the we'd just get new Vec<u8>'s when/if we run out in the producer
                        if log_buffer.capacity() > buffer_pool_max_capacity {
                            let log_was_oversized = log_buffer.len() > buffer_pool_max_capacity;

                            if log_was_oversized {
                                oversized_messages_count += 1;

                                // Check how many in the last N hours
                                let percentage_of_oversized = oversized_messages_count as f64
                                    / total_messages_count as f64
                                    * 100.0;

                                // If 50% of our logs are oversized, give a warning every 1 in 5 logs
                                // small minimum of 1000 to keep noise down right from the beginning
                                let warn_every_n = match percentage_of_oversized {
                                    p if p > 50.0 && total_messages_count > 1000 => 5,
                                    p if p > 30.0 && total_messages_count > 1000 => 10,
                                    _ => 100, // Default to 1% of logs (1 in 100)
                                };

                                // Log a warning on first ocurrance or every N (set above)
                                if oversized_messages_count == 1
                                    || oversized_messages_count.is_multiple_of(warn_every_n)
                                {
                                    // If its an oversized string, we obviously don't want to log the whole thing
                                    let preview_len =
                                        log_buffer.len().min(OVERSIZED_LOG_PREVIEW_LENGTH);
                                    let oversized_log_preview = // get the actual text
                                    String::from_utf8_lossy(&log_buffer[..preview_len]);

                                    let truncation_note =
                                        if log_buffer.len() > OVERSIZED_LOG_PREVIEW_LENGTH {
                                            format!("... ({} bytes total)", log_buffer.len())
                                        } else {
                                            String::new()
                                        };
                                    eprintln!(
                                        "SJL_WARN: You have logs that are greater than your buffer_pool_max_capacity ({} bytes). \
                                    This log was {} bytes. Right now {percentage_of_oversized:.2}% of total logs are oversized. \
                                    Consider increasing the buffer_pool_initial_capacity value if you see this log a lot. \
                                    Log that triggered this: {oversized_log_preview}{truncation_note}",
                                        buffer_pool_max_capacity,
                                        log_buffer.len()
                                    )
                                }
                            }
                        }

                        // Clear the buffer
                        log_buffer.clear();
                        // And shrink any that has grown past the max threshold since they double each time
                        log_buffer.shrink_to(buffer_pool_initial_capacity);
                        // and return it to the pool
                        let _ = buffer_pool.push(log_buffer);

                        // Happy path, flush logs
                        if batch_message_count >= flush_at_messages || batch.len() >= flush_at_bytes
                        {
                            Logger::flush(&mut batch);
                            batch_message_count = 0;
                        }
                    }
                    // Flush regardless of what happened, we might be shutting down
                    Err(RecvTimeoutError::Disconnected) => {
                        Logger::flush(&mut batch);
                        break;
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        Logger::flush(&mut batch);
                        batch_message_count = 0;
                        // Don't break to keep the loop going
                    }
                }
            }
        })
    }

    fn flush(batch: &mut Vec<u8>) {
        if batch.is_empty() {
            return;
        }

        let mut stdout = std::io::stdout().lock();

        let _ = stdout.write_all(batch);
        let _ = stdout.flush();

        batch.clear();
    }
}
