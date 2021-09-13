/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use std::fs:File;
use std::include_str;
use std::process::{Command, ExitStatus, Stdio};
use tempfile::tempfile;

pub struct AnsiblePlaybook
{
    file_contents: String
}

impl AnsiblePlaybook
{
    pub fn load(file_contents: &str) -> AnsiblePlaybook
    {
        AnsiblePlaybook {
            file_contents: file_contents
        }
    }

    pub fn get_update_command() -> AnsiblePlaybook
    {
        load(include_str!("playbooks/update.yaml"))
    }

    pub fn run(&self) -> ExitStatus
    {
        // Save file to disk
        let temp_file_name = tempfile().expect("Failed to get a temporary file");
        File::create(temp_file_name)
            .write_all(self.file_contents)
            .expect("Failed saving playbook to temporary file");

        // Run playbook
        let output = Command::new("ansible-playbook")
            .args([temp_file_name])
            .output()
            .expect("Failed to execute ansible-playbook");
        return output.status;
    }
}