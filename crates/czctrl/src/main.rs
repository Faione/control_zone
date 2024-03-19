use anyhow::Result;
use clap::{Args, Parser};
use log::error;

mod commands;
mod config;
mod control_zone;

#[derive(Parser, Debug)]
enum SubCommand {
    #[clap(flatten)]
    Advance(Box<commands::AdvanceCmd>),

    #[clap(flatten)]
    Basic(Box<commands::BasicCmd>),
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Opts {
    #[clap(flatten)]
    global_opts: GloablOpts,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Debug, Args)]
struct GloablOpts {
    /// just print the results
    #[clap(short, long, global = true)]
    dry_run: bool,
}

fn main() -> Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let opts = Opts::parse();
    let cmd_result = match opts.subcmd {
        SubCommand::Advance(cmd) => match *cmd {
            commands::AdvanceCmd::Apply(apply) => commands::apply::apply(apply),
            commands::AdvanceCmd::Down(down) => commands::down::down(down),
            commands::AdvanceCmd::Observe(observe) => commands::observe::observe(observe),
            commands::AdvanceCmd::List(list) => commands::list::list(list),
            _ => todo!(),
        },

        SubCommand::Basic(cmd) => match *cmd {
            commands::BasicCmd::Create(create) => {
                commands::create::create(create, &opts.global_opts)
            }
            _ => todo!(),
        },
    };

    if let Err(ref e) = cmd_result {
        error!("error in executing command: {e:?}");
    }

    cmd_result
}
