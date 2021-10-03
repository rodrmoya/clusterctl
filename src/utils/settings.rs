/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use super::logging::LogLevel;

pub struct ClusterSettings
{
    pub inventory_file: String,
    pub verbosity_level: LogLevel
}

impl ClusterSettings
{
    pub fn log(&self, level: LogLevel, msg: &str)
    {
        let print_it = match self.verbosity_level {
            LogLevel::Trace => true,
            LogLevel::Debug => !matches!(level, LogLevel::Trace),
            _ => matches!(level, LogLevel::Info)
        };
        if print_it {
            println!("[{}]: {}", level.as_ref(), msg);
        }
    }
}