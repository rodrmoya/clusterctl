/*
 * CLI to manage a cluster of machines.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use strum_macros::{AsRefStr, EnumString};

#[derive(Debug, AsRefStr, EnumString)]
pub enum LogLevel {
    Info,
    Debug,
    Trace,
}