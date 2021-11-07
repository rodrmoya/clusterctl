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

    pub fn with_optional_parameter(self, param_name: &str, param_value: &Option<String>) -> Self {
        match param_value {
            Some(v) => self.with_parameter(param_name, &v),
            None => self
        }
    }

    pub fn run(&self, settings: &ClusterSettings) -> Result<ExitStatus, Error> {
        let mut args: Vec<String> = vec![
            // Inventory file to use
            "--inventory".to_string(),
            settings.inventory.clone()
        ];

        if self.needs_become {
            args.push("-K".to_string());
            args.push("-b".to_string());
        }

        // Command to run
        args.push("-m".to_string());
        args.push(self.command.clone());

        // And now all extra parameters
        if self.parameters.len() > 0 {
            for param in &self.parameters {
                if !param.0.is_empty() && !param.1.is_empty() {
                    args.push(format!("-a {}=\"{}\"", param.0, param.1));
                }
            }
        }

        args.push("all".to_string());

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

    pub fn save_to_file(&self) -> String {
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
        let file_name = playbook.save_to_file();
        args.push(file_name);
    }

    // Run playbook
    info!("Executing Ansible playbooks");
    Command::new("ansible-playbook")
        .stdin(Stdio::piped())
        .args(args)
        .status()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use rstest::rstest;
    use crate::commands::{INSTALL_DOCKER_COMMAND_PLAYBOOK, INSTALL_KUBERNETES_COMMAND_PLAYBOOK, SETUP_KUBERNETES_CLUSTER_COMMAND_PLAYBOOK, UNINSTALL_DOCKER_COMMAND_PLAYBOOK, UNINSTALL_KUBERNETES_COMMAND_PLAYBOOK};
    use super::{AnsibleCommand, AnsiblePlaybook};

    #[rstest]
    #[case(INSTALL_DOCKER_COMMAND_PLAYBOOK)]
    #[case(INSTALL_KUBERNETES_COMMAND_PLAYBOOK)]
    #[case(SETUP_KUBERNETES_CLUSTER_COMMAND_PLAYBOOK)]
    #[case(UNINSTALL_DOCKER_COMMAND_PLAYBOOK)]
    #[case(UNINSTALL_KUBERNETES_COMMAND_PLAYBOOK)]
    fn playbook_is_correctly_saved(
        #[case] playbook_contents: &str) {
            let playbook = AnsiblePlaybook::load(playbook_contents);
            assert_eq!(playbook.file_contents, playbook_contents);

            let playbook_file = playbook.save_to_file();
            let saved_playbook_contents = fs::read_to_string(playbook_file).unwrap();
            assert_eq!(saved_playbook_contents, playbook.file_contents);
    }

    #[rstest]
    #[case(None, false)]
    #[case(None, true)]
    #[case(Some(String::new()), false)]
    #[case(Some(String::new()), true)]
    fn commands_with_parameters_are_correctly_built(
        #[case] optional_parameter: Option<String>,
        #[case] needs_become: bool) {
        let command = AnsibleCommand::new("my_command", needs_become)
            .with_parameter("param1", "param1_value")
            .with_parameter("param2", "param2_value")
            .with_optional_parameter("opt_param1", &optional_parameter);

        assert_eq!(command.command, "my_command");
        assert_eq!(command.needs_become, needs_become);
        assert_eq!(command.parameters.get("param1").unwrap(), "param1_value");
        assert_eq!(command.parameters.get("param2").unwrap(), "param2_value");

        if let Some(v) = optional_parameter {
            assert!(command.parameters.contains_key("opt_param1"));
            assert_eq!(command.parameters.get("opt_param1").unwrap(), &v);
        } else {
            assert!(!command.parameters.contains_key("opt_param1"));
        }
    }
}