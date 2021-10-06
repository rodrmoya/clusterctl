/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

 use std::include_str;
use clap::Clap;

mod ansible;
pub use ansible::AnsiblePlaybook;

use super::ClusterSettings;

// Command names, which are also playbook file names
const REBOOT_COMMAND_PLAYBOOK: &str = include_str!("../../playbooks/reboot.yaml");
const UPDATE_COMMAND_PLAYBOOK: &str = include_str!("../../playbooks/update.yaml");

#[derive(Clap)]
pub struct RebootCommand;

#[derive(Clap)]
pub struct ServiceCommand
{
    #[clap(subcommand)]
    pub subcommand: ServiceSubCommand
}

#[derive(Clap)]
pub enum ServiceSubCommand
{
    Deploy(DeployServiceCommand)
}

#[derive(Clap)]
pub struct DeployServiceCommand
{
    pub service: String
}

#[derive(Clap)]
pub struct UpdateCommand;

pub fn run_reboot(settings: &ClusterSettings, rc: &RebootCommand) -> i32
{
    AnsiblePlaybook::load(REBOOT_COMMAND_PLAYBOOK)
        .run(settings)
}

pub fn run_service(settings: &ClusterSettings, sc: &ServiceCommand) -> i32
{
    return match &sc.subcommand {
        ServiceSubCommand::Deploy(ref dsc) => run_deploy_service(settings, sc, dsc)
    };
}

fn run_deploy_service(settings: &ClusterSettings, sc: &ServiceCommand, dsc: &DeployServiceCommand) -> i32
{
    -1
}

pub fn run_update(settings: &ClusterSettings, uc: &UpdateCommand) -> i32
{
    AnsiblePlaybook::load(UPDATE_COMMAND_PLAYBOOK)
        .run(settings)
}