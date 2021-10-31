/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use std::include_str;

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

pub fn run_reboot(settings: &ClusterSettings, rc: &RebootCommand) -> i32
{
    AnsiblePlaybook::load(REBOOT_COMMAND_PLAYBOOK)
        .run(settings)
}

pub fn run_service(settings: &ClusterSettings, sc: &ServiceCommand) -> i32
{
    return match &sc.subcommand {
        ServiceSubCommand::Deploy(ref dsc) => run_deploy_service(settings, sc, dsc),
        ServiceSubCommand::Delete(ref dsc) => run_delete_service(settings, sc, dsc)
    };
}

fn run_deploy_service(settings: &ClusterSettings, sc: &ServiceCommand, dsc: &ServiceCommandOptions) -> i32
{
    let mut playbook = AnsibleAggregatePlaybook::new();

    settings.log_info(format!("Deploying service '{}' to cluster", &dsc.service).as_str());
    if dsc.service == "kubernetes" {
        playbook.add_playbook(AnsiblePlaybook::load(INSTALL_KUBERNETES_COMMAND_PLAYBOOK));
        playbook.add_playbook(AnsiblePlaybook::load(SETUP_KUBERNETES_CLUSTER_COMMAND_PLAYBOOK));
    } else {
        eprintln!("Unknown service '{}', can't deploy", dsc.service);
        return -1;
    }

    return playbook.run(settings);
}

fn run_delete_service(settings: &ClusterSettings, sc: &ServiceCommand, dsc: &ServiceCommandOptions) -> i32
{
    let mut playbook = AnsibleAggregatePlaybook::new();

    settings.log_info(format!("Deleting service '{}' from cluster", &dsc.service).as_str());
    if dsc.service == "kubernetes" {
        playbook.add_playbook(AnsiblePlaybook::load(UNINSTALL_KUBERNETES_COMMAND_PLAYBOOK));
    } else {
        eprintln!("Unknown service '{}', can't deploy", dsc.service);
        return -1;
    }

    return playbook.run(settings);
}

pub fn run_update(settings: &ClusterSettings, uc: &UpdateCommand) -> i32
{
    AnsiblePlaybook::load(UPDATE_COMMAND_PLAYBOOK)
        .run(settings)
}