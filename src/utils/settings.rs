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
    pub inventory: Option<String>,

    #[clap(short, long, parse(from_occurrences), about = "Level of verbosity")]
    pub verbose: u64,

    #[clap(short = 'p', long, about = "Host pattern. If not specified, all machines in the cluster is assumed")]
    pub host_pattern: Option<String>,

    #[clap(subcommand)]
    pub subcommand: SubCommand
}

#[derive(Clap, Debug)]
pub enum SubCommand {
    #[clap(about = "Copy local files to machines in the cluster")]
    Copy(CopyCommand),
    #[clap(about = "Fetch files from machines in the cluster")]
    Fetch(CopyCommand),
    #[clap(about = "Commands to operate on the configured inventory")]
    Inventory(InventoryCommand),
    #[clap(about = "Ping all machines in the cluster to check they're alive and reachable")]
    Ping(GenericCommand),
    #[clap(about = "Reboot all machines in the cluster")]
    Reboot(GenericCommand),
    #[clap(about = "Run a command on all machines in the cluster")]
    Run(RunCommand),
    #[clap(about = "Commands to operate services on the cluster")]
    Service(ServiceCommand),
    #[clap(about = "Shut down machines in the cluster")]
    Shutdown(GenericCommand),
    #[clap(about = "Open a secure shell connection to a machine on the cluster")]
    Ssh(GenericCommand),
    #[clap(about = "Perform OS and apps updates on all the machines in the cluster")]
    Update(GenericCommand),
    #[clap(about = "Show how long machines in the cluster have been running")]
    Uptime(GenericCommand)
}

#[derive(Clap, Debug)]
pub struct GenericCommand;

#[derive(Clap, Debug)]
pub struct CopyCommand {
    #[clap(long, about = "Specifify source file on the remote or local machine")]
    pub src: String,

    #[clap(long, about = "Specifify destination file on the remote or local machine")]
    pub dest: String
}

#[derive(Clap, Debug)]
pub struct InventoryCommand {
    #[clap(subcommand)]
    pub subcommand: InventorySubCommand
}

#[derive(Clap, Debug)]
pub enum InventorySubCommand {
    #[clap(about = "List all configured hosts in the inventory")]
    List(InventoryCommandOptions)
}

#[derive(Clap, Debug)]
pub struct InventoryCommandOptions;

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

        assert_eq!(settings.inventory.unwrap(), INVENTORY_FILE);

        let verbosity_level = log::max_level();
        assert!(matches!(verbosity_level, expected_verbosity));
    }

    #[rstest]
    #[case("clusterctl --inventory /tmp/inventory.yaml ping", SubCommand::Ping(GenericCommand))]
    #[case("clusterctl --inventory /tmp/inventory.yaml update", SubCommand::Update(GenericCommand))]
    #[case("clusterctl --inventory /tmp/inventory.yaml reboot", SubCommand::Reboot(GenericCommand))]
    #[case("clusterctl --inventory /tmp/inventory.yaml reboot", SubCommand::Shutdown(GenericCommand))]
    #[case("clusterctl --inventory /tmp/inventory.yaml ssh", SubCommand::Ssh(GenericCommand))]
    #[case("clusterctl --inventory /tmp/inventory.yaml uptime", SubCommand::Uptime(GenericCommand))]
    fn command_and_options_are_correctly_parsed(
        #[case] command_line: String,
        #[case] expected_subcommand: SubCommand) {
        let args: Vec<&str> = command_line.split(' ').collect();
        let settings: ClusterSettings = ClusterSettings::try_parse_from(args).unwrap();

        assert_eq!(settings.inventory.unwrap(), INVENTORY_FILE);
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

        assert_eq!(settings.inventory.unwrap(), INVENTORY_FILE);

        if let SubCommand::Run(ref rc) = settings.subcommand {
            assert_eq!(rc.command, expected_command);
            assert_eq!(rc.chdir, expected_directory);
            assert_eq!(rc.needs_become, expected_needs_become);
        } else {
            panic!("Command {:?} is wrong", settings.subcommand);
        }
    }

    #[rstest]
    #[case("clusterctl --inventory /tmp/inventory.yaml copy --src=/tmp/file --dest=/tmp/", "/tmp/file", "/tmp/")]
    #[case("clusterctl --inventory /tmp/inventory.yaml fetch --src=/tmp/file --dest=/tmp/", "/tmp/file", "/tmp/")]
    fn copy_command_and_options_are_correctly_parsed(
        #[case] command_line: &str,
        #[case] expected_src: &str,
        #[case] expected_dest: &str) {
        let args: Vec<&str> = command_line.split(' ').collect();
        let settings: ClusterSettings = ClusterSettings::try_parse_from(args).unwrap();

        assert_eq!(settings.inventory.unwrap(), INVENTORY_FILE);

        match settings.subcommand {
            SubCommand::Copy(cc) | SubCommand::Fetch(cc) => {
                assert_eq!(expected_src, cc.src);
                assert_eq!(expected_dest, cc.dest);
            },
            _ => panic!("Command {:?} is wrong", settings.subcommand)
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

        assert_eq!(settings.inventory.unwrap(), INVENTORY_FILE);

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

        assert_eq!(settings.inventory.unwrap(), INVENTORY_FILE);

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