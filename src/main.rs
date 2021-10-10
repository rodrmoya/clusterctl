/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moy

    use crate::utils::logging::LogLevel;
    use crate::utils::logging::LogLevel;
a <rodrigo@gnome.org>
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

#[cfg(test)]
mod tests
{
    use clap::Clap;
    use rstest::rstest;
    use crate::ClusterSettings;
    use crate::utils::logging::LogLevel;

    const INVENTORY_FILE: &str = "/tmp/inventory.yaml";

    #[rstest]
    #[case("-v", LogLevel::Debug)]
    #[case("-vv", LogLevel::Trace)]
    #[case("-vvv", LogLevel::Trace)]
    fn global_settings_are_correctly_parsed(
        #[case] verbosity_arg: &str,
        #[case] expected_verbosity: LogLevel)
    {
        let settings: ClusterSettings = ClusterSettings::try_parse_from(
            vec!["pi-cluster", verbosity_arg, "--inventory", INVENTORY_FILE, "update"]
        ).unwrap();

        assert_eq!(settings.inventory, INVENTORY_FILE);

        let verbosity_level = settings.verbosity_level();
        matches!(verbosity_level, expected_verbosity);
    }
}
