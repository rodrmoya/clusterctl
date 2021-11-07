/*
 * CLI to manage a cluster of machines.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use clap::{Clap, crate_version, crate_authors, crate_description};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
pub struct ClusterSettings {
    #[clap(short, long, about = "Host inventory file (in Ansible supported format)")]
    pub inventory: String,

    #[clap(short, long, parse(from_occurrences), about = "Level of verbosity")]
    pub verbose: u64,

    #[clap(short = 'p', long, about = "Host pattern. If not specified, all machines in the cluster is assumed")]
    pub host_pattern: Option<String>,

    #[clap(subcommand)]
    pub subcommand: SubCommand
}

#[derive(Clap, Debug)]
pub enum SubCommand {
    #[clap(about = "Ping all machines in the cluster to check they're alive and reachable")]
    Ping(PingCommand),
    #[clap(about = "Reboot all machines in the cluster")]
    Reboot(RebootCommand),
    #[clap(about = "Run a command on all machines in the cluster")]
    Run(RunCommand),
    #[clap(about = "Commands to operate services on the cluster")]
    Service(ServiceCommand),
    #[clap(about = "Open a secure shell connection to a machine on the cluster")]
    Ssh(SshCommand),
    #[clap(about = "Perform OS and apps updates on all the machines in the cluster")]
    Update(UpdateCommand)
}

#[derive(Clap, Debug)]
pub struct PingCommand;

#[derive(Clap, Debug)]
pub struct RebootCommand;

#[derive(Clap, Debug)]
pub struct RunCommand {
    pub command: String,

    #[clap(short, long, about = "Specifies the command needs to be run with elevated privileges")]
    pub needs_become: bool,

    #[clap(short, long, about = "Directory on the cluster machines to chdir to before running the command")]
    pub chdir: Option<String>
}

#[derive(Clap, Debug)]
pub struct ServiceCommand {
    #[clap(subcommand)]
    pub subcommand: ServiceSubCommand
}

#[derive(Clap, Debug)]
pub enum ServiceSubCommand {
    #[clap(about = "Deploy a service on the cluster")]
    Deploy(ServiceCommandOptions),
    #[clap(about = "Delete a service from the cluster")]
    Delete(ServiceCommandOptions)
}

#[derive(Clap, Debug)]
pub struct ServiceCommandOptions {
    #[clap(about = "Service name ('kubernetes', 'docker', ...)")]
    pub service: String
}

#[derive(Clap, Debug)]
pub struct SshCommand;

#[derive(Clap, Debug)]
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
    #[case("clusterctl --inventory /tmp/inventory.yaml ssh", SubCommand::Ssh(SshCommand))]
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

            if let SubCommand::Run(ref rc) = settings.subcommand {
                assert_eq!(rc.command, expected_command);
                assert_eq!(rc.chdir, expected_directory);
                assert_eq!(rc.needs_become, expected_needs_become);
            } else {
                panic!("Command {:?} is wrong", settings.subcommand);
            }
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

            if let SubCommand::Service(ref sc) = settings.subcommand {
                if let ServiceSubCommand::Deploy(ref ssc) = sc.subcommand {
                    assert_eq!(ssc.service, expected_service_name);
                } else {
                    panic!("Subcommand {:?} is wrong", sc.subcommand);
                }
            } else {
                panic!("Command {:?} is wrong", settings.subcommand);
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

            if let SubCommand::Service(ref sc) = settings.subcommand {
                if let ServiceSubCommand::Delete(ref ssc) = sc.subcommand {
                    assert_eq!(ssc.service, expected_service_name);
                } else {
                    panic!("Subcommand {:?} is wrong", sc.subcommand);
                }
            } else {
                panic!("Command {:?} is wrong", settings.subcommand);
            }
    }
}