/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use std::include_str;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

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

    pub fn run(&self) -> i32
    {
        // Save file to disk
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", self.file_contents)
            .expect("Failed saving playbook to temporary file");
        
        // Run playbook
        let output = Command::new("ansible-playbook")
            .args([temp_file.path()])
            .output()
            .expect("Failed to execute ansible-playbook");

        if output.status.success() {
            return 0;
        }

        return -1;
    }
}