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
            SubCommand::Copy(ref cc) => copy_or_fetch_file(self, "copy", cc),
            SubCommand::Fetch(ref cc) => copy_or_fetch_file(self, "fetch", cc),
            SubCommand::Ping(ref _gc) => run_simple_command(self, "ping", false),
            SubCommand::Reboot(ref _gc) => run_simple_command(self, "reboot", true),
            SubCommand::Run(ref rc) => run_command_in_cluster(self, &rc.command, rc.needs_become, &rc.chdir),
            SubCommand::Service(ref sc) => run_service(self, sc),
            SubCommand::Shutdown(ref _gc) => run_simple_command(self, "community.general.shutdown", true),
            SubCommand::Ssh(ref _sc) => run_ssh(self),
            SubCommand::Update(ref _gc) => run_update(self),
            SubCommand::Uptime(ref _uc) => run_command_in_cluster(self, "uptime", false, &Option::<String>::None)
        }
    }
}

fn copy_or_fetch_file(settings: &ClusterSettings, ansible_cmd: &str, cc: &CopyCommand) -> Result<ExitStatus, Error> {
    AnsibleCommand::new(ansible_cmd, false, settings.host_pattern.clone())
        .with_parameter("src", cc.src.as_str())
        .with_parameter("dest", cc.dest.as_str())
        .run(settings)
}

fn run_simple_command(settings: &ClusterSettings, cmd: &str, needs_become: bool) -> Result<ExitStatus, Error> {
    AnsibleCommand::new(cmd, needs_become, settings.host_pattern.clone())
        .run(settings)
}

fn run_command_in_cluster(settings: &ClusterSettings, command: &str, needs_become: bool, chdir: &Option<String>) -> Result<ExitStatus, Error> {
    AnsibleCommand::new(&String::new(), needs_become, settings.host_pattern.clone())
        .with_parameter(command, &String::new())
        .with_optional_parameter("chdir", chdir)
        .run(settings)
}

fn run_service(settings: &ClusterSettings, sc: &ServiceCommand) -> Result<ExitStatus, Error> {
    match &sc.subcommand {
        ServiceSubCommand::Deploy(ref dsc) => run_deploy_service(settings, sc, dsc),
        ServiceSubCommand::Delete(ref dsc) => run_delete_service(settings, sc, dsc)
    }
}

fn run_deploy_service(settings: &ClusterSettings, _sc: &ServiceCommand, dsc: &ServiceCommandOptions) -> Result<ExitStatus, Error> {
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

fn run_delete_service(settings: &ClusterSettings, _sc: &ServiceCommand, dsc: &ServiceCommandOptions) -> Result<ExitStatus, Error> {
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

    playbook.run(settings)
}

fn run_ssh(settings: &ClusterSettings) -> Result<ExitStatus, Error> {
    AnsibleCommand::new("ssh", false, settings.host_pattern.clone())
        .run(settings)
}

fn run_update(settings: &ClusterSettings) -> Result<ExitStatus, Error> {
    AnsibleCommand::new("apt", true, settings.host_pattern.clone())
        .with_parameter("update_cache", "yes")
        .with_parameter("autoremove", "yes")
        .with_parameter("force_apt_get", "yes")
        .with_parameter("upgrade", "yes")
        .run(settings)
}