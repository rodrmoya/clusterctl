/*
 * CLI to manage a cluster of machines.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use std::include_str;
use std::io::{Error, ErrorKind};
use std::process::ExitStatus;

use log::{error, info};

mod ansible;
use crate::commands::ansible::{AnsibleAggregatePlaybook, AnsibleCommand, AnsiblePlaybook};

use crate::utils::settings::*;

// Command names, which are also playbook file names
const INSTALL_KUBERNETES_COMMAND_PLAYBOOK: &str = include_str!("../../playbooks/install-kubernetes.yaml");
const UNINSTALL_KUBERNETES_COMMAND_PLAYBOOK: &str = include_str!("../../playbooks/uninstall-kubernetes.yaml");
const SETUP_KUBERNETES_CLUSTER_COMMAND_PLAYBOOK: &str = include_str!("../../playbooks/setup-kubernetes-cluster.yaml");

const INSTALL_DOCKER_COMMAND_PLAYBOOK: &str = include_str!("../../playbooks/install-docker.yaml");
const UNINSTALL_DOCKER_COMMAND_PLAYBOOK: &str = include_str!("../../playbooks/uninstall-docker.yaml");

// Service names
const SERVICE_NAME_DOCKER: &str = "docker";
const SERVICE_NAME_KUBERNETES: &str = "kubernetes";

pub trait CommandRunner {
    fn run(&self) -> Result<ExitStatus, Error>;
}

impl CommandRunner for ClusterSettings {
    fn run(&self) -> Result<ExitStatus, Error>
    {
        match self.subcommand {
            SubCommand::Copy(ref cc) => {
                AnsibleCommand::new_copy_command(false, self.host_pattern.clone(), cc.src.as_str(), cc.dest.as_str())
                    .run(self)
            },
            SubCommand::Fetch(ref cc) => {
                AnsibleCommand::new_fetch_command(false, self.host_pattern.clone(), cc.src.as_str(), cc.dest.as_str())
                    .run(self)
            },
            SubCommand::Inventory(ref lc) => {
                match &lc.subcommand {
                    &InventorySubCommand::List(ref _options) => {
                        ansible::list_hosts(self)
                    }
                }
            },
            SubCommand::Ping(ref _gc) => {
                AnsibleCommand::new("ping", false, self.host_pattern.clone())
                    .run(self)
            },
            SubCommand::Reboot(ref _gc) => {
                AnsibleCommand::new("reboot", true, self.host_pattern.clone())
                    .run(self)
            },
            SubCommand::Run(ref rc) => {
                AnsibleCommand::new_run_command(&rc.command, rc.needs_become, self.host_pattern.clone(), rc.chdir.clone())
                    .run(self)
            },
            SubCommand::Service(ref sc) => {
                match &sc.subcommand {
                    ServiceSubCommand::Deploy(ref options) => run_deploy_service(self, sc, options),
                    ServiceSubCommand::Delete(ref options) => run_delete_service(self, sc, options)
                }
            },
            SubCommand::Shutdown(ref _gc) => {
                AnsibleCommand::new("community.general.shutdown", true, self.host_pattern.clone())
                    .run(self)
            },
            SubCommand::Ssh(ref _sc) => {
                AnsibleCommand::new("ssh", false, self.host_pattern.clone())
                    .run(self)
            },
            SubCommand::Update(ref _gc) => {
                AnsibleCommand::new_update_command(self.host_pattern.clone())
                    .run(self)
            },
            SubCommand::Uptime(ref _uc) => {
                AnsibleCommand::new_run_command("uptime", false, self.host_pattern.clone(), Option::<String>::None)
                    .run(self)
            }
        }
    }
}

fn run_deploy_service(settings: &ClusterSettings, _sc: &ServiceCommand, options: &ServiceCommandOptions) -> Result<ExitStatus, Error> {
    let mut playbook = AnsibleAggregatePlaybook::new();

    info!("Deploying service '{}' to cluster", &options.service);
    if options.service == SERVICE_NAME_KUBERNETES {
        playbook.add_playbook(AnsiblePlaybook::load(INSTALL_KUBERNETES_COMMAND_PLAYBOOK));
        playbook.add_playbook(AnsiblePlaybook::load(SETUP_KUBERNETES_CLUSTER_COMMAND_PLAYBOOK));
    } else if options.service == SERVICE_NAME_DOCKER {
        playbook.add_playbook(AnsiblePlaybook::load(INSTALL_DOCKER_COMMAND_PLAYBOOK));
    } else {
        let msg = format!("Unknown service '{}', can't deploy", options.service);
        error!("{}", msg);
        return Err(Error::new(ErrorKind::Other, msg));
    }

    playbook.run(settings)
}

fn run_delete_service(settings: &ClusterSettings, _sc: &ServiceCommand, options: &ServiceCommandOptions) -> Result<ExitStatus, Error> {
    let mut playbook = AnsibleAggregatePlaybook::new();

    info!("Deleting service '{}' from cluster", &options.service);
    if options.service == SERVICE_NAME_KUBERNETES {
        playbook.add_playbook(AnsiblePlaybook::load(UNINSTALL_KUBERNETES_COMMAND_PLAYBOOK));
    } else if options.service == SERVICE_NAME_DOCKER {
        playbook.add_playbook(AnsiblePlaybook::load(UNINSTALL_DOCKER_COMMAND_PLAYBOOK));
    } else {
        let msg = format!("Unknown service '{}', can't deploy", options.service);
        error!("{}", msg);
        return Err(Error::new(ErrorKind::Other, msg));
    }

    playbook.run(settings)
}