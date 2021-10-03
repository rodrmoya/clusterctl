/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use super::ansible::AnsiblePlaybook;
use crate::utils::settings::ClusterSettings;

pub fn run_update(settings: &ClusterSettings) -> i32
{
    AnsiblePlaybook::get_update_command()
        .run(settings)
}