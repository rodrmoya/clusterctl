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

#[cfg(test)]
mod tests {
    use clap::Clap;
    use log::Level;
    use rstest::rstest;
    use crate::utils::settings::*;

    const INVENTORY_FILE: &str = "/tmp/inventory.yaml";

    #[rstest]
    #[case("-v", Level::Info)]
    #[case("-vv", Level::Trace)]
    #[case("-vvv", Level::Trace)]
    fn global_settings_are_correctly_parsed(
        #[case] verbosity_arg: &str,
        #[case] expected_verbosity: Level) {
        let settings: ClusterSettings = ClusterSettings::try_parse_from(
            vec!["clusterctl", verbosity_arg, "--inventory", INVENTORY_FILE, "update"]
        ).unwrap();

        assert_eq!(settings.inventory, INVENTORY_FILE);

        let verbosity_level = log::max_level();
        assert!(matches!(verbosity_level, expected_verbosity));
    }

    #[rstest]
    #[case("clusterctl --inventory /tmp/inventory.yaml ping", SubCommand::Ping(PingCommand))]
    #[case("clusterctl --inventory /tmp/inventory.yaml update", SubCommand::Update(UpdateCommand))]
    #[case("clusterctl --inventory /tmp/inventory.yaml reboot", SubCommand::Reboot(RebootCommand))]
    fn command_and_options_are_correctly_parsed(
        #[case] command_line: String,
        #[case] expected_subcommand: SubCommand) {
            let args: Vec<&str> = command_line.split(' ').collect();
            let settings: ClusterSettings = ClusterSettings::try_parse_from(args).unwrap();

            assert_eq!(settings.inventory, INVENTORY_FILE);
            assert!(matches!(settings.subcommand, expected_subcommand));
    }

    #[rstest]
    #[case("clusterctl --inventory /tmp/inventory.yaml run ls --chdir /", "ls", Some("/".to_string()), false)]
    #[case("clusterctl --inventory /tmp/inventory.yaml run ls --needs-become --chdir /", "ls", Some("/".to_string()), true)]
    #[case("clusterctl --inventory /tmp/inventory.yaml run ls", "ls", None, false)]
    #[case("clusterctl --inventory /tmp/inventory.yaml run ls --needs-become", "ls", None, true)]
    fn run_command_and_options_are_correctly_parsed(
        #[case] command_line: &str,
        #[case] expected_command: &str,
        #[case] expected_directory: Option<String>,
        #[case] expected_needs_become: bool) {
            let args: Vec<&str> = command_line.split(' ').collect();
            let settings: ClusterSettings = ClusterSettings::try_parse_from(args).unwrap();

            assert_eq!(settings.inventory, INVENTORY_FILE);

            match settings.subcommand {
                SubCommand::Run(ref rc) => {
                    assert_eq!(rc.command, expected_command);
                    assert_eq!(rc.chdir, expected_directory);
                    assert_eq!(rc.needs_become, expected_needs_become);
                },
                _ => assert!(false),
            };
    }

    #[rstest]
    #[case("clusterctl --inventory /tmp/inventory.yaml service deploy kubernetes", "kubernetes")]
    #[case("clusterctl --inventory /tmp/inventory.yaml service deploy docker", "docker")]
    fn service_deploy_command_and_options_are_correctly_parsed(
        #[case] command_line: &str,
        #[case] expected_service_name: &str) {
            let args: Vec<&str> = command_line.split(' ').collect();
            let settings: ClusterSettings = ClusterSettings::try_parse_from(args).unwrap();

            assert_eq!(settings.inventory, INVENTORY_FILE);

            match settings.subcommand {
                SubCommand::Service(ref sc) => {
                    match sc.subcommand {
                        ServiceSubCommand::Deploy(ref ssc) => {
                            assert_eq!(ssc.service, expected_service_name);
                        }
                        _ => assert!(false),
                    }
                },
                _ => assert!(false),
            }
    }

    #[rstest]
    #[case("clusterctl --inventory /tmp/inventory.yaml service delete kubernetes", "kubernetes")]
    #[case("clusterctl --inventory /tmp/inventory.yaml service delete docker", "docker")]
    fn service_delete_command_and_options_are_correctly_parsed(
        #[case] command_line: &str,
        #[case] expected_service_name: &str) {
            let args: Vec<&str> = command_line.split(' ').collect();
            let settings: ClusterSettings = ClusterSettings::try_parse_from(args).unwrap();

            assert_eq!(settings.inventory, INVENTORY_FILE);

            match settings.subcommand {
                SubCommand::Service(ref sc) => {
                    match sc.subcommand {
                        ServiceSubCommand::Delete(ref ssc) => {
                            assert_eq!(ssc.service, expected_service_name);
                        }
                        _ => assert!(false),
                    }
                },
                _ => assert!(false),
            }
    }
}