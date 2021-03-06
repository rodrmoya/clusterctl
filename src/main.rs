/*
 * CLI to manage a cluster of machines.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use std::io::Error;

use clap::Clap;
use simple_logger::SimpleLogger;

mod utils;
use utils::settings::ClusterSettings;
mod commands;
use commands::CommandRunner;

fn main() -> Result<(), Error> {
    SimpleLogger::new().init().unwrap();

    let settings: ClusterSettings = ClusterSettings::parse();
    settings.run()?;

    Ok(())
}
