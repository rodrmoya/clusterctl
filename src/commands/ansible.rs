/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use std::include_str;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;
use crate::utils::logging::LogLevel;
use crate::ClusterSettings;

pub struct AnsiblePlaybook
{
    file_contents: String
}

impl AnsiblePlaybook
{
    pub fn load(file_contents: &str) -> AnsiblePlaybook
    {
        AnsiblePlaybook {
            file_contents: String::from(file_contents)
        }
    }

    pub fn get_playbook_for_command(cmd: &str) -> AnsiblePlaybook
    {
        if cmd == "reboot" {
            return AnsiblePlaybook::load(include_str!("../../playbooks/reboot.yaml"));
        } else if cmd == "update" {
            return AnsiblePlaybook::load(include_str!("../../playbooks/update.yaml"));
        }

        AnsiblePlaybook {
            file_contents: String::new()
        }
    }

    pub fn run(&self, settings: &ClusterSettings) -> i32
    {
        // Save file to disk
        let mut temp_file = NamedTempFile::new()
            .expect("Could not create temp file");
        settings.log_trace(format!("Writing Ansible playbook to {}", temp_file.path().display()).as_str());
        temp_file.write_all(self.file_contents.as_bytes())
            .expect("Failed saving playbook to temporary file");
        
        // Run playbook
        settings.log_info("Executing Ansible playbook");
        let output = Command::new("ansible-playbook")
            .stdin(Stdio::piped())
            .args([
                "-K",
                "--inventory",
                settings.inventory.as_str(),
                temp_file.path().to_str().unwrap()
            ])
            .status()
            .expect("Failed to execute ansible-playbook");

        if output.success() {
            return 0;
        }

        return -1;
    }
}