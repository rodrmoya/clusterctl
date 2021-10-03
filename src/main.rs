/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use clap::{App, crate_name, crate_version, crate_authors, crate_description, SubCommand};

pub mod commands;

fn main()
{
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        //.arg(Arg::with_name("config").short("c").value_name("FILE").takes_value(true))
        .subcommand(SubCommand::with_name("update")
            .about("Update OS and apps on the whole cluster"))
        .get_matches();

    let mut exit_code: i32 = -1;
    if let Some(ref matches) = matches.subcommand_matches("update") {
         exit_code = commands::run_update();
    }

    std::process::exit(exit_code);
}
