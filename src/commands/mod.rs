/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use std::include_str;
use std::io::{Error, ErrorKind};
use std::process::ExitStatus;

mod ansible;
pub use ansible::AnsiblePlaybook;

use crate::commands::ansible::AnsibleAggregatePlaybook;

use crate::utils::settings::*;

// Command names, which are also playbook file names
const REBOOT_COMMAND_PLAYBOOK: &str = include_str!("../../playbooks/reboot.yaml");
const UPDATE_COMMAND_PLAYBOOK: &str = include_str!("../../playbooks/update.yaml");

const INSTALL_KUBERNETES_COMMAND_PLAYBOOK: &str = include_str!("../../playbooks/install-kubernetes.yaml");
const UNINSTALL_KUBERNETES_COMMAND_PLAYBOOK: &str = include_str!("../../playbooks/uninstall-kubernetes.yaml");
const SETUP_KUBERNETES_CLUSTER_COMMAND_PLAYBOOK: &str = include_str!("../../playbooks/setup-kubernetes-cluster.yaml");

pub fn run_reboot(settings: &ClusterSettings, rc: &RebootCommand) -> Result<ExitStatus, Error>
{
    AnsiblePlaybook::load(REBOOT_COMMAND_PLAYBOOK)
        .run(settings)
}

pub fn run_service(settings: &ClusterSettings, sc: &ServiceCommand) -> Result<ExitStatus, Error>
{
    return match &sc.subcommand {
        ServiceSubCommand::Deploy(ref dsc) => run_deploy_service(settings, sc, dsc),
        ServiceSubCommand::Delete(ref dsc) => run_delete_service(settings, sc, dsc)
    };
}

fn run_deploy_service(settings: &ClusterSettings, sc: &ServiceCommand, dsc: &ServiceCommandOptions) -> Result<ExitStatus, Error>
{
    let mut playbook = AnsibleAggregatePlaybook::new();

    settings.log_info(format!("Deploying service '{}' to cluster", &dsc.service).as_str());
    if dsc.service == "kubernetes" {
        playbook.add_playbook(AnsiblePlaybook::load(INSTALL_KUBERNETES_COMMAND_PLAYBOOK));
        playbook.add_playbook(AnsiblePlaybook::load(SETUP_KUBERNETES_CLUSTER_COMMAND_PLAYBOOK));
    } else {
        let msg = format!("Unknown service '{}', can't deploy", dsc.service);
        eprintln!("{}", msg);
        return Err(Error::new(ErrorKind::Other, msg));
    }

    playbook.run(settings)
}

fn run_delete_service(settings: &ClusterSettings, sc: &ServiceCommand, dsc: &ServiceCommandOptions) -> Result<ExitStatus, Error>
{
    let mut playbook = AnsibleAggregatePlaybook::new();

    settings.log_info(format!("Deleting service '{}' from cluster", &dsc.service).as_str());
    if dsc.service == "kubernetes" {
        playbook.add_playbook(AnsiblePlaybook::load(UNINSTALL_KUBERNETES_COMMAND_PLAYBOOK));
    } else {
        let msg = format!("Unknown service '{}', can't deploy", dsc.service);
        eprintln!("{}", msg);
        return Err(Error::new(ErrorKind::Other, msg));
    }

    return playbook.run(settings);
}

pub fn run_update(settings: &ClusterSettings, uc: &UpdateCommand) -> Result<ExitStatus, Error>
{
    AnsiblePlaybook::load(UPDATE_COMMAND_PLAYBOOK)
        .run(settings)
}