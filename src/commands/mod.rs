/*
 * CLI to manage a cluster of machines.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use std::include_str;
use std::io::{Error, ErrorKind};
use std::process::ExitStatus;

use log::{info, error};

mod ansible;
pub use ansible::AnsiblePlaybook;
use crate::commands::ansible::{AnsibleAggregatePlaybook, AnsibleCommand};

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
            SubCommand::Ping(ref pc) => run_simple_command(self, "ping", false),
            SubCommand::Reboot(ref rc) => run_simple_command(self, "reboot", true),
            SubCommand::Service(ref sc) => run_service(self, sc),
            SubCommand::Update(ref uc) => run_update(self, uc)
        }
    }
}

fn run_simple_command(settings: &ClusterSettings, cmd: &str, needs_become: bool) -> Result<ExitStatus, Error> {
    AnsibleCommand::new(cmd, needs_become)
        .run(settings)
}

fn run_service(settings: &ClusterSettings, sc: &ServiceCommand) -> Result<ExitStatus, Error> {
    match &sc.subcommand {
        ServiceSubCommand::Deploy(ref dsc) => run_deploy_service(settings, sc, dsc),
        ServiceSubCommand::Delete(ref dsc) => run_delete_service(settings, sc, dsc)
    }
}

fn run_deploy_service(settings: &ClusterSettings, sc: &ServiceCommand, dsc: &ServiceCommandOptions) -> Result<ExitStatus, Error> {
    let mut playbook = AnsibleAggregatePlaybook::new();

    info!("Deploying service '{}' to cluster", &dsc.service);
    if dsc.service == SERVICE_NAME_KUBERNETES {
        playbook.add_playbook(AnsiblePlaybook::load(INSTALL_KUBERNETES_COMMAND_PLAYBOOK));
        playbook.add_playbook(AnsiblePlaybook::load(SETUP_KUBERNETES_CLUSTER_COMMAND_PLAYBOOK));
    } else if dsc.service == SERVICE_NAME_DOCKER {
        playbook.add_playbook(AnsiblePlaybook::load(INSTALL_DOCKER_COMMAND_PLAYBOOK));
    } else {
        let msg = format!("Unknown service '{}', can't deploy", dsc.service);
        error!("{}", msg);
        return Err(Error::new(ErrorKind::Other, msg));
    }

    playbook.run(settings)
}

fn run_delete_service(settings: &ClusterSettings, sc: &ServiceCommand, dsc: &ServiceCommandOptions) -> Result<ExitStatus, Error> {
    let mut playbook = AnsibleAggregatePlaybook::new();

    info!("Deleting service '{}' from cluster", &dsc.service);
    if dsc.service == SERVICE_NAME_KUBERNETES {
        playbook.add_playbook(AnsiblePlaybook::load(UNINSTALL_KUBERNETES_COMMAND_PLAYBOOK));
    } else if dsc.service == SERVICE_NAME_DOCKER {
        playbook.add_playbook(AnsiblePlaybook::load(UNINSTALL_DOCKER_COMMAND_PLAYBOOK));
    } else {
        let msg = format!("Unknown service '{}', can't deploy", dsc.service);
        error!("{}", msg);
        return Err(Error::new(ErrorKind::Other, msg));
    }

    return playbook.run(settings);
}

fn run_update(settings: &ClusterSettings, uc: &UpdateCommand) -> Result<ExitStatus, Error> {
    AnsibleCommand::new("apt", true)
        .with_parameter("update_cache", "yes")
        .with_parameter("autoremove", "yes")
        .with_parameter("force_apt_get", "yes")
        .with_parameter("upgrade", "yes")
        .run(settings)
}