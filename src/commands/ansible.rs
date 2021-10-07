/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;
use crate::ClusterSettings;

// Represents a single Ansible playbook
pub struct AnsiblePlaybook
{
    file_contents: String
}

pub struct AnsibleAggregatePlaybook
{
    playbooks: Vec<AnsiblePlaybook>
}

impl AnsiblePlaybook
{
    pub fn load(file_contents: &str) -> AnsiblePlaybook
    {
        AnsiblePlaybook {
            file_contents: String::from(file_contents)
        }
    }

    pub fn save_to_file(&self, settings: &ClusterSettings) -> String
    {
        let mut temp_file = NamedTempFile::new()
            .expect("Could not create temp file");
        settings.log_trace(format!("Writing Ansible playbook to {}", temp_file.path().display()).as_str());
        temp_file.write_all(self.file_contents.as_bytes())
            .expect("Failed saving playbook to temporary file");

        return temp_file.path().to_string_lossy().to_string();
    }

    pub fn run(&self, settings: &ClusterSettings) -> i32
    {
        // Save file to disk
        let temp_file = self.save_to_file(settings);
        
        // Run playbook
        settings.log_info("Executing Ansible playbook");
        let output = Command::new("ansible-playbook")
            .stdin(Stdio::piped())
            .args([
                "-K",
                "--inventory",
                settings.inventory.as_str(),
                &temp_file
            ])
            .status()
            .expect("Failed to execute ansible-playbook");

        if output.success() {
            return 0;
        }

        return -1;
    }
}

impl AnsibleAggregatePlaybook
{
    pub fn new() -> AnsibleAggregatePlaybook
    {
        AnsibleAggregatePlaybook {
            playbooks: Vec::new()
        }
    }

    pub fn add_playbook(&mut self, playbook: AnsiblePlaybook)
    {
        self.playbooks.push(playbook);
    }

    pub fn run(&self, settings: &ClusterSettings) -> i32
    {
        let mut args: Vec<String> = vec![
            "-K".to_string(),
            "--inventory".to_string(),
            settings.inventory.clone()
            ];

        for playbook in &self.playbooks {
            let file_name = playbook.save_to_file(settings);
            args.push(file_name);
        }

        // Run playbook
        settings.log_info("Executing Ansible playbooks");
        let output = Command::new("ansible-playbook")
            .stdin(Stdio::piped())
            .args(args)
            .status()
            .expect("Failed to execute ansible-playbook");

        if output.success() {
            return 0;
        }

        return -1;
    }
}