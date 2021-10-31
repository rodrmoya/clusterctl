/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use clap::{Clap, crate_version, crate_authors, crate_description};

use super::logging::LogLevel;

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
pub struct ClusterSettings
{
    #[clap(short, long)]
    pub inventory: String,

    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u64,

    #[clap(subcommand)]
    pub subcommand: SubCommand
}

#[derive(Clap)]
pub enum SubCommand
{
    Reboot(RebootCommand),
    Service(ServiceCommand),
    Update(UpdateCommand)
}

#[derive(Clap)]
pub struct RebootCommand;

#[derive(Clap)]
pub struct ServiceCommand
{
    #[clap(subcommand)]
    pub subcommand: ServiceSubCommand
}

#[derive(Clap)]
pub enum ServiceSubCommand
{
    Deploy(ServiceCommandOptions),
    Delete(ServiceCommandOptions)
}

#[derive(Clap)]
pub struct ServiceCommandOptions
{
    pub service: String
}

#[derive(Clap)]
pub struct UpdateCommand;

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