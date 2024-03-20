use anyhow::Result;
use clap::{Args, Parser};
use log::error;

mod commands;
mod config;
mod controlzone;

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
            commands::AdvanceCmd::Observe(observe) => {
                commands::observe::observe(observe, &opts.global_opts)
            }
            commands::AdvanceCmd::List(list) => commands::list::list(list),
        },

        SubCommand::Basic(cmd) => match *cmd {
            commands::BasicCmd::Create(create) => {
                commands::create::create(create, &opts.global_opts)
            }
            commands::BasicCmd::Start(start) => commands::start::start(start, &opts.global_opts),
            commands::BasicCmd::Stop(stop) => commands::stop::stop(stop, &opts.global_opts),
            commands::BasicCmd::Remove(remove) => {
                commands::remove::remove(remove, &opts.global_opts)
            }
            commands::BasicCmd::Update(update) => {
                commands::update::update(update, &opts.global_opts)
            }

            commands::BasicCmd::Inspect(inspect) => {
                commands::inspect::inspect(inspect, &opts.global_opts)
            }
        },
    };

    if let Err(ref e) = cmd_result {
        error!("error in executing command: {e:?}");
    }

    cmd_result
}
