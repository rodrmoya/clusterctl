/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use clap::{App, Arg, crate_name, crate_version, crate_authors, crate_description, SubCommand};

pub mod commands;
use commands::ClusterSettings;

fn main()
{
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("verbose")
            .short("v")
            .multiple(true)
            .help("verbosity level"))
        .arg(Arg::with_name("inventory")
            .long("inventory")
            .short("i")
            .value_name("Hosts file (in Ansible format)")
            .takes_value(true))
        .subcommand(SubCommand::with_name("update")
            .about("Update OS and apps on the whole cluster"))
        .get_matches();

    let mut exit_code: i32 = -1;
    if let Some(ref inventory_file) = matches.value_of("inventory") {
        let settings = ClusterSettings {
            inventory_file: inventory_file.to_string(),
            verbosity_level: match matches.occurrences_of("v") {
                0 => commands::LogLevel::Info,
                1 => commands::LogLevel::Debug,
                2 | _ => commands::LogLevel::Trace
            }
        };

        if let Some(ref matches) = matches.subcommand_matches("update") {
            exit_code = commands::run_update(&settings);
        }
    } else {
        eprintln!("Inventory file not specified, please specify it via the --inventory option");
    }

    std::process::exit(exit_code);
}
