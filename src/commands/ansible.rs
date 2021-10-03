/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use std::include_str;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;
use crate::utils::logging::LogLevel;
use crate::utils::settings::ClusterSettings;

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

    pub fn get_update_command() -> AnsiblePlaybook
    {
        AnsiblePlaybook::load(include_str!("../../playbooks/update.yaml"))
    }

    pub fn run(&self, settings: &ClusterSettings) -> i32
    {
        // Save file to disk
        let mut temp_file = NamedTempFile::new()
            .expect("Could not create temp file");
        settings.log(LogLevel::Trace, format!("Writing Ansible playbook to {}", temp_file.path().display()).as_str());
        temp_file.write_all(self.file_contents.as_bytes())
            .expect("Failed saving playbook to temporary file");
        
        // Run playbook
        settings.log(LogLevel::Info, "Executing Ansible playbook");
        let output = Command::new("ansible-playbook")
            .args([
                "--inventory",
                settings.inventory_file.as_str(),
                temp_file.path().to_str().unwrap()
            ])
            .output()
            .expect("Failed to execute ansible-playbook");

        settings.log(LogLevel::Info, format!("{}", String::from_utf8_lossy(&output.stdout)).as_str());
        if output.status.success() {
            return 0;
        }

        return -1;
    }
}