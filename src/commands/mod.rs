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
pub struct UpdateCommand;

pub fn run_reboot(settings: &ClusterSettings) -> i32
{
    AnsiblePlaybook::get_playbook_for_command("reboot")
        .run(settings)
}

pub fn run_update(settings: &ClusterSettings) -> i32
{
    AnsiblePlaybook::get_playbook_for_command("update")
        .run(settings)
}