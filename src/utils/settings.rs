/*
 * CLI to manage a cluster of machines.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use clap::{Clap, crate_version, crate_authors, crate_description};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
pub struct ClusterSettings {
    #[clap(short, long)]
    pub inventory: String,

    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u64,

    #[clap(subcommand)]
    pub subcommand: SubCommand
}

#[derive(Clap)]
pub enum SubCommand {
    Ping(PingCommand),
    Reboot(RebootCommand),
    Run(RunCommand),
    Service(ServiceCommand),
    Update(UpdateCommand)
}

#[derive(Clap)]
pub struct PingCommand;

#[derive(Clap)]
pub struct RebootCommand;

#[derive(Clap)]
pub struct RunCommand {
    pub command: String,

    #[clap(short, long)]
    pub needs_become: bool,

    #[clap(short, long)]
    pub chdir: Option<String>
}

#[derive(Clap)]
pub struct ServiceCommand {
    #[clap(subcommand)]
    pub subcommand: ServiceSubCommand
}

#[derive(Clap)]
pub enum ServiceSubCommand {
    Deploy(ServiceCommandOptions),
    Delete(ServiceCommandOptions)
}

#[derive(Clap)]
pub struct ServiceCommandOptions {
    pub service: String
}

#[derive(Clap)]
pub struct UpdateCommand;