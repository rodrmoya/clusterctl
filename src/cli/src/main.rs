use clap::{App, crate_name, crate_version, crate_authors, crate_description, SubCommand};

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        //.arg(Arg::with_name("config").short("c").value_name("FILE").takes_value(true))
        .subcommand(SubCommand::with_name("init")
            .about("Initialize cluster"))
        .subcommand(SubCommand::with_name("cleanup"))
        .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("init") {
    } else if let Some(ref matches) = matches.subcommand_matches("cleanup") {
    }
}
