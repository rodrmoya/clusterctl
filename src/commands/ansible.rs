/*
 * CLI to manage a cluster of machines.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use std::collections::HashMap;
use std::convert::TryInto;
use std::io::{Error, Write};
use std::process::{Command, ExitStatus, Stdio};
use tempfile::NamedTempFile;
use log::info;
use crate::utils::settings::ClusterSettings;

/// Lists all hosts and groups in the configured inventory file.
pub fn list_hosts(settings: &ClusterSettings) -> Result<ExitStatus, Error> {
    let command_arguments = {
        let mut args: Vec<String> = Vec::new();

        args.push("--graph".to_string());
        args.push("--vars".to_string());
        if let Some(v) = &settings.inventory {
            args.push("--inventory".to_string());
            args.push(v.clone());
        }

        args
    };

    Command::new("ansible-inventory")
        .stdin(Stdio::piped())
        .args(command_arguments)
        .status()
}

/// Represents an Ansible command "session"
pub struct AnsibleCommand {
    command: String,
    needs_become: bool,
    host_pattern: Option<String>,
    parameters: HashMap<String, String>
}

impl AnsibleCommand {
    pub fn new(command: &str, needs_become: bool, host_pattern: Option<String>) -> Self {
        AnsibleCommand {
            command: command.to_string(),
            needs_become,
            host_pattern,
            parameters: HashMap::new()
        }
    }

    /// Creates a new `AnsibleCommand` instance for copying local files to remote machines.
    pub fn new_copy_command(
        needs_become: bool,
        host_pattern: Option<String>,
        src: &str,
        dest: &str) -> AnsibleCommand {
        AnsibleCommand::new("copy", needs_become, host_pattern)
            .with_parameter("src", src)
            .with_parameter("dest", dest)
    }

    /// Creates a new `AnsibleCommand` instance for fetching files from remote machines.
    pub fn new_fetch_command(
        needs_become: bool,
        host_pattern: Option<String>,
        src: &str,
        dest: &str) -> AnsibleCommand {
        AnsibleCommand::new("fetch", needs_become, host_pattern)
            .with_parameter("src", src)
            .with_parameter("dest", dest)
    }

    pub fn new_run_command(command: &str, needs_become: bool, host_pattern: Option<String>, chdir: Option<String>) -> AnsibleCommand {
        AnsibleCommand::new(&String::new(), needs_become, host_pattern.clone())
            .with_parameter(command, &String::new())
            .with_optional_parameter("chdir", &chdir)
    }

    /// Creates a new `AnsibleCommand` instance for updating remore machines.
    pub fn new_update_command(host_pattern: Option<String>) -> AnsibleCommand {
        AnsibleCommand::new("apt", true, host_pattern.clone())
            .with_parameter("update_cache", "yes")
            .with_parameter("autoremove", "yes")
            .with_parameter("force_apt_get", "yes")
            .with_parameter("upgrade", "yes")
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
        let command_arguments = {
            let mut args: Vec<String> = Vec::new();

            if let Some(v) = get_verbose_arguments_from_settings(settings) {
                args.push(v);
            }

            if let Some(v) = &settings.inventory {
                args.push("--inventory".to_string());
                args.push(v.clone());
            }

            if self.needs_become {
                args.push("-K".to_string());
                args.push("-b".to_string());
            }

            // Command to run
            if !self.command.is_empty() {
                args.push("-m".to_string());
                args.push(self.command.clone());
            }

            // And now all extra parameters
            if self.parameters.len() > 0 {
                let mut action_args = String::new();
                for param in &self.parameters {
                    if !param.0.is_empty() && !param.1.is_empty() {
                        if action_args.is_empty() {
                            action_args.push_str("-a ");
                        }
                        action_args.push_str(&format!("{}=\"{}\" ", param.0, param.1));
                    } else if !param.0.is_empty() && param.1.is_empty() {
                        if action_args.is_empty() {
                            action_args.push_str("-a ");
                        }
                        action_args.push_str(&format!("{} ", param.0.to_string()));
                    }
                }

                if !action_args.is_empty() {
                    args.push(action_args);
                }
            }

            if let Some(pattern) = &self.host_pattern {
                args.push(pattern.clone());
            } else {
                args.push("all".to_string());
            }

            args
        };

        info!("Executing Ansible command {} {:?}", self.command.clone(), command_arguments);
        Command::new("ansible")
            .stdin(Stdio::piped())
            .args(command_arguments)
            .status()
    }
}

/// Represents a single Ansible playbook
pub struct AnsiblePlaybook {
    file_contents: String
}

impl AnsiblePlaybook {
    #[cfg(test)]
    pub fn get_available_playbooks() -> Vec<AnsiblePlaybook> {
        use crate::commands::{INSTALL_DOCKER_COMMAND_PLAYBOOK, INSTALL_KUBERNETES_COMMAND_PLAYBOOK, SETUP_KUBERNETES_CLUSTER_COMMAND_PLAYBOOK, UNINSTALL_DOCKER_COMMAND_PLAYBOOK, UNINSTALL_KUBERNETES_COMMAND_PLAYBOOK};

        vec![
            AnsiblePlaybook::load(INSTALL_DOCKER_COMMAND_PLAYBOOK),
            AnsiblePlaybook::load(INSTALL_KUBERNETES_COMMAND_PLAYBOOK),
            AnsiblePlaybook::load(SETUP_KUBERNETES_CLUSTER_COMMAND_PLAYBOOK),
            AnsiblePlaybook::load(UNINSTALL_DOCKER_COMMAND_PLAYBOOK),
            AnsiblePlaybook::load(UNINSTALL_KUBERNETES_COMMAND_PLAYBOOK)
        ]
    }

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

    #[cfg(test)]
    pub fn check_syntax(&self) -> Result<ExitStatus, Error> {
        let playbook_file = self.save_to_file();
        let command_arguments = vec![
            "--syntax-check",
            &playbook_file
        ];

        Command::new("ansible-playbook")
            .stdin(Stdio::piped())
            .args(command_arguments)
            .status()
    }
}

/// Represents a set of Ansible playbooks to be run together
pub struct AnsibleAggregatePlaybook {
    playbooks: Vec<AnsiblePlaybook>
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
    let command_arguments = {
        let mut args: Vec<String> = Vec::new();

        if let Some(v) = get_verbose_arguments_from_settings(settings) {
            args.push(v);
        }

        args.push("-K".to_string());

        if let Some(v) = &settings.inventory {
            args.push("--inventory".to_string());
            args.push(v.clone());
        }

        for playbook in &playbooks {
            let file_name = playbook.save_to_file();
            args.push(file_name);
        }

        args
    };

    // Run playbook
    info!("Executing Ansible playbooks");
    Command::new("ansible-playbook")
        .stdin(Stdio::piped())
        .args(command_arguments)
        .status()
}

fn get_verbose_arguments_from_settings(settings: &ClusterSettings) -> Option<String> {
    match settings.verbose {
        0 => None,
        count if count >= 1 && count <= 4 => Some(format!("-{}", str::repeat("v", count.try_into().unwrap()))),
        _ => None
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, process::ExitStatus};
    use rstest::rstest;
    use super::{AnsibleCommand, AnsiblePlaybook};

    #[rstest]
    fn playbook_is_correctly_saved() {
        for playbook in AnsiblePlaybook::get_available_playbooks() {
            let playbook_file = playbook.save_to_file();
            let saved_playbook_contents = fs::read_to_string(playbook_file).unwrap();
            assert_eq!(saved_playbook_contents, playbook.file_contents);
        }
    }

    #[rstest]
    fn playbooks_syntax_is_correct() {
        for playbook in AnsiblePlaybook::get_available_playbooks() {
            let syntax_check_result = playbook.check_syntax();
            assert!(ExitStatus::success(&syntax_check_result.unwrap()));
        }
    }

    #[rstest]
    #[case(None, false)]
    #[case(None, true)]
    #[case(Some(String::new()), false)]
    #[case(Some(String::new()), true)]
    fn commands_with_parameters_are_correctly_built(
        #[case] optional_parameter: Option<String>,
        #[case] needs_become: bool) {
        let command = AnsibleCommand::new("my_command", needs_become, None)
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