/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use std::io::Error;

use clap::Clap;
pub mod commands;
pub mod utils;
use utils::settings::{ClusterSettings, SubCommand};

fn main() -> Result<(), Error>
{
    let settings: ClusterSettings = ClusterSettings::parse();

    if !settings.inventory.is_empty() {
        match settings.subcommand {
            SubCommand::Reboot(ref rc) => commands::run_reboot(&settings, rc)?,
            SubCommand::Service(ref sc) => commands::run_service(&settings, sc)?,
            SubCommand::Update(ref uc) => commands::run_update(&settings, uc)?
        };
    } else {
        eprintln!("Inventory file not specified, please specify it via the --inventory option");
    }

    Ok(())
}

#[cfg(test)]
mod tests
{
    use clap::Clap;
    use rstest::rstest;
    use crate::utils::settings::ClusterSettings;
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
        assert!(matches!(verbosity_level, expected_verbosity));
    }
}
