/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use super::ansible::AnsiblePlaybook;

pub fn run_update(inventory_file: &str) -> i32
{
    AnsiblePlaybook::get_update_command()
        .run(inventory_file)
}