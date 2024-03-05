use anyhow::Result;
use clap::Parser;
use log::error;

mod commands;

#[derive(Parser, Debug)]
enum SubCommand {
    #[clap(flatten)]
    ControlZone(Box<commands::ControlZoneCmd>),
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

fn main() -> Result<()> {
    env_logger::init();
    let opts = Opts::parse();
    let cmd_result = match opts.subcmd {
        SubCommand::ControlZone(cmd) => match *cmd {
            commands::ControlZoneCmd::Apply(apply) => commands::apply::apply(apply),
            commands::ControlZoneCmd::Down(down) => commands::down::down(down),
            commands::ControlZoneCmd::Generate(generate) => commands::generate::generate(generate),
            commands::ControlZoneCmd::Observe(observe) => commands::observe::observe(observe),
        },
    };

    if let Err(ref e) = cmd_result {
        error!("error in executing command: {:?}", e);
    }

    cmd_result
}
