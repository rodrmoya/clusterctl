/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use clap::{Clap, crate_version, crate_authors, crate_description};

pub mod commands;
pub mod utils;
use commands::{RebootCommand, ServiceCommand, UpdateCommand};

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

fn main()
{
    let settings: ClusterSettings = ClusterSettings::parse();

    let mut exit_code: i32 = -1;
    if !settings.inventory.is_empty() {
        exit_code = match settings.subcommand {
            SubCommand::Reboot(ref rc) => commands::run_reboot(&settings, rc),
            SubCommand::Service(ref sc) => commands::run_service(&settings, sc),
            SubCommand::Update(ref uc) => commands::run_update(&settings, uc)
        }
    } else {
        eprintln!("Inventory file not specified, please specify it via the --inventory option");
    }

    std::process::exit(exit_code);
}
