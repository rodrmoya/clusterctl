/*
 * CLI to manage a cluster of machines.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use std::io::Error;

use clap::Clap;
pub mod commands;
pub mod utils;
use utils::settings::ClusterSettings;
use commands::CommandRunner;

fn main() -> Result<(), Error> {
    let settings: ClusterSettings = ClusterSettings::parse();

    settings.run()?;
    Ok(())
}

#[cfg(test)]
mod tests {
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
        #[case] expected_verbosity: LogLevel) {
        let settings: ClusterSettings = ClusterSettings::try_parse_from(
            vec!["clusterctl", verbosity_arg, "--inventory", INVENTORY_FILE, "update"]
        ).unwrap();

        assert_eq!(settings.inventory, INVENTORY_FILE);

        let verbosity_level = settings.verbosity_level();
        assert!(matches!(verbosity_level, expected_verbosity));
    }
}
