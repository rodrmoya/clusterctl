/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

mod ansible;
pub use ansible::AnsiblePlaybook;

mod update_command;
pub use update_command::run_update;

pub enum LogLevel
{
    Info,
    Debug,
    Trace
}

pub struct ClusterSettings
{
    pub inventory_file: String,
    pub verbosity_level: LogLevel
}