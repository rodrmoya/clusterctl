/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use super::logging::LogLevel;
use crate::ClusterSettings;

impl ClusterSettings
{
    pub fn log(&self, level: LogLevel, msg: &str)
    {
        let print_it = match self.verbosity_level() {
            LogLevel::Trace => true,
            LogLevel::Debug => !matches!(level, LogLevel::Trace),
            _ => matches!(level, LogLevel::Info)
        };
        if print_it {
            println!("[{}]: {}", level.as_ref(), msg);
        }
    }

    pub fn log_info(&self, msg: &str)
    {
        self.log(LogLevel::Info, msg);
    }

    pub fn log_debug(&self, msg: &str)
    {
        self.log(LogLevel::Debug, msg);
    }

    pub fn log_trace(&self, msg: &str)
    {
        self.log(LogLevel::Trace, msg);
    }

    pub fn verbosity_level(&self) -> LogLevel
    {
        return match self.verbose {
            0 => LogLevel::Info,
            1 => LogLevel::Debug,
            2 | _ => LogLevel::Trace
        };
    }
}