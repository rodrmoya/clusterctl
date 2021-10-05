/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use clap::Clap;

mod ansible;
pub use ansible::AnsiblePlaybook;

use super::ClusterSettings;

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
pub struct DeployServiceCommand;

#[derive(Clap)]
pub struct UpdateCommand;

pub fn run_reboot(settings: &ClusterSettings, rc: &RebootCommand) -> i32
{
    AnsiblePlaybook::get_playbook_for_command("reboot")
        .run(settings)
}

pub fn run_service(settings: &ClusterSettings, sc: &ServiceCommand) -> i32
{
    return match &sc.subcommand {
        ServiceSubCommand::Deploy(dsc) => -1
    };
}

pub fn run_update(settings: &ClusterSettings, uc: &UpdateCommand) -> i32
{
    AnsiblePlaybook::get_playbook_for_command("update")
        .run(settings)
}