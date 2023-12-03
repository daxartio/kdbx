use std::io;

use clap::{Command, CommandFactory};
use clap_complete::{generate, Generator, Shell};

use crate::{Cli, Result};

#[derive(clap::Args)]
pub struct Args {
    #[arg(short, long, value_enum)]
    shell: Shell,
}

pub(crate) fn run(args: Args) -> Result<()> {
    let mut cmd = Cli::command();
    eprintln!(
        "Generating completion file for {shell:?}...",
        shell = args.shell
    );
    print_completions(args.shell, &mut cmd);
    Ok(())
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
