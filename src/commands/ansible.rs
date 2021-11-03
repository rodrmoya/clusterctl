/*
 * CLI to manage a cluster of machines.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use std::collections::HashMap;
use std::io::{Error, Write};
use std::process::{Command, ExitStatus, Stdio};
use tempfile::NamedTempFile;
use log::info;
use crate::ClusterSettings;

// Represents an Ansible command "session"
pub struct AnsibleCommand {
    command: String,
    needs_become: bool,
    parameters: HashMap<String, String>
}

// Represents a single Ansible playbook
pub struct AnsiblePlaybook {
    file_contents: String
}

pub struct AnsibleAggregatePlaybook {
    playbooks: Vec<AnsiblePlaybook>
}

impl AnsibleCommand {
    pub fn new(command: &str, needs_become: bool) -> Self {
        AnsibleCommand {
            command: command.to_string(),
            needs_become,
            parameters: HashMap::new()
        }
    }

    pub fn with_parameter(mut self, param_name: &str, param_value: &str) -> Self {
        self.parameters.insert(param_name.to_string(), param_value.to_string());
        self
    }

    pub fn run(&self, settings: &ClusterSettings) -> Result<ExitStatus, Error> {
        let mut args: Vec<String> = vec![
            // Inventory file to use
            "--inventory".to_string(),
            settings.inventory.clone(),
            "all".to_string()
        ];

        if self.needs_become {
            args.push("-K".to_string());
        }

        // Command to run
        args.push("-m".to_string());
        args.push(self.command.clone());

        info!("Executing Ansible command {}", self.command.clone());
        Command::new("ansible")
            .stdin(Stdio::piped())
            .args(args)
            .status()
    }
}

impl AnsiblePlaybook {
    pub fn load(file_contents: &str) -> AnsiblePlaybook {
        AnsiblePlaybook {
            file_contents: String::from(file_contents)
        }
    }

    pub fn save_to_file(&self, settings: &ClusterSettings) -> String {
        let mut temp_file = NamedTempFile::new()
            .expect("Could not create temp file");
        info!("Writing Ansible playbook to {}", temp_file.path().display());
        temp_file.write_all(self.file_contents.as_bytes())
            .expect("Failed saving playbook to temporary file");
        let path = temp_file.path().to_string_lossy().to_string();
        temp_file.keep()
            .expect("Could not persist temporary file");

        path
    }

    pub fn run(&self, settings: &ClusterSettings) -> Result<ExitStatus, Error> {
        run_ansible_playbook(settings, vec![self])
    }
}

impl AnsibleAggregatePlaybook {
    pub fn new() -> AnsibleAggregatePlaybook {
        AnsibleAggregatePlaybook {
            playbooks: Vec::new()
        }
    }

    pub fn add_playbook(&mut self, playbook: AnsiblePlaybook) {
        self.playbooks.push(playbook);
    }

    pub fn run(&self, settings: &ClusterSettings) -> Result<ExitStatus, Error> {
        let mut playbooks: Vec<&AnsiblePlaybook> = Vec::new();
        for playbook in &self.playbooks {
            playbooks.push(&playbook);
        }
        run_ansible_playbook(settings, playbooks)
    }
}

fn run_ansible_playbook(settings: &ClusterSettings, playbooks: Vec<&AnsiblePlaybook>) -> Result<ExitStatus, Error> {
    let mut args: Vec<String> = vec![
        // Ask for password for sudo
        "-K".to_string(),
        // Inventory file to use
        "--inventory".to_string(),
        settings.inventory.clone()
        ];

    for playbook in &playbooks {
        let file_name = playbook.save_to_file(settings);
        args.push(file_name);
    }

    // Run playbook
    info!("Executing Ansible playbooks");
    Command::new("ansible-playbook")
        .stdin(Stdio::piped())
        .args(args)
        .status()
}