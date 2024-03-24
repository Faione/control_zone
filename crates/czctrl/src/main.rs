use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser};
use libcz::WORKDIR_ROOT;
use log::error;
use vruntime::VRuntimeType;

mod commands;
mod config;
mod pod;
mod vruntime;

#[derive(Parser, Debug)]
enum SubCommand {
    #[clap(flatten)]
    Advance(Box<commands::AdvanceCmd>),

    #[clap(flatten)]
    Basic(Box<commands::BasicCmd>),

    /// Manage Pod of Control Zone
    #[clap(subcommand)]
    Pod(Box<pod::PodCmd>),
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
    #[arg(short, long, global = true)]
    dry_run: bool,

    #[arg(long, global = true)]
    root: Option<PathBuf>,

    #[arg(long, value_enum, default_value_t = VRuntimeType::Libvirt ,global = true)]
    vruntime: VRuntimeType,
}

impl GloablOpts {
    #[inline]
    fn root_dir(&self) -> PathBuf {
        match &self.root {
            Some(path) => path.to_owned(),
            None => PathBuf::from(WORKDIR_ROOT),
        }
    }
}

fn main() -> Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let opts = Opts::parse();
    let cmd_result = match opts.subcmd {
        SubCommand::Advance(cmd) => match *cmd {
            commands::AdvanceCmd::Apply(apply) => commands::apply::apply(apply, &opts.global_opts),
            commands::AdvanceCmd::Down(down) => commands::down::down(down, &opts.global_opts),
            commands::AdvanceCmd::Observe(observe) => {
                commands::observe::observe(observe, &opts.global_opts)
            }
            commands::AdvanceCmd::List(list) => commands::list::list(list, &opts.global_opts),
            commands::AdvanceCmd::Conn(conn) => commands::conn::conn(conn, &opts.global_opts),
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
            commands::BasicCmd::Log(log) => commands::log::log(log, &opts.global_opts),
        },
        SubCommand::Pod(cmd) => match *cmd {
            pod::PodCmd::Add(add) => pod::add::add(add, &opts.global_opts),
            pod::PodCmd::Delete(delete) => pod::delete::delete(delete, &opts.global_opts),
            pod::PodCmd::Show(show) => pod::show::show(show, &opts.global_opts),
        },
    };

    if let Err(ref e) = cmd_result {
        error!("error in executing command: {e:?}");
    }

    cmd_result
}
