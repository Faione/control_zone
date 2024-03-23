use clap::{Parser, Subcommand, ValueEnum};
use libc::{
    sched_get_priority_max, sched_get_priority_min, sched_param, EINVAL, ENAVAIL, EPERM, ESRCH,
};

#[cfg(not(any(target_env = "musl", target_os = "emscripten", target_env = "ohos")))]
use libc::{sched_getparam, sched_getscheduler, sched_setscheduler};

#[cfg(any(target_env = "musl", target_os = "emscripten", target_env = "ohos"))]
use libc::{syscall, SYS_sched_getparam, SYS_sched_getscheduler, SYS_sched_setscheduler};

mod sched;

use sched::SCHED;

#[derive(Subcommand)]
enum Commands {
    #[command(visible_alias = "r")]
    Read,

    #[command(visible_alias = "w")]
    Write(Write),
}

#[derive(Parser, Debug)]
struct Write {
    /// Sched Policy
    #[arg(short, long, required = true)]
    sched: SCHED,

    /// Priority if exist, else zero
    prio: Option<i32>,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short, long, global = true, required = false)]
    pid: i32,

    #[command(subcommand)]
    command: Commands,
}

fn get_scheduler(pid: i32) -> String {
    #[cfg(any(target_env = "musl", target_os = "emscripten", target_env = "ohos"))]
    let pol = unsafe { syscall(SYS_sched_getscheduler, pid) as i32 };

    #[cfg(not(any(target_env = "musl", target_os = "emscripten", target_env = "ohos")))]
    let pol = unsafe { sched_getscheduler(pid) };

    if pol < 0 {
        return String::from("INVALID");
    }

    let sched_class: SCHED = pol.into();
    sched_class
        .to_possible_value()
        .map(|pv| String::from(pv.get_name()))
        .unwrap()
}

fn main() {
    let cli = Cli::parse();
    let pid = cli.pid;

    #[cfg(any(target_env = "musl", target_os = "emscripten", target_env = "ohos"))]
    let mut param: sched_param = sched_param {
        sched_priority: 0,
        sched_ss_low_priority: 0,
        sched_ss_repl_period: libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        sched_ss_init_budget: libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        sched_ss_max_repl: 0,
    };

    #[cfg(not(any(target_env = "musl", target_os = "emscripten", target_env = "ohos")))]
    let mut param: sched_param = sched_param { sched_priority: 0 };

    unsafe {
        #[cfg(any(target_env = "musl", target_os = "emscripten", target_env = "ohos"))]
        let ret = syscall(SYS_sched_getparam, pid, &mut param as *mut sched_param) as i32;

        #[cfg(not(any(target_env = "musl", target_os = "emscripten", target_env = "ohos")))]
        let ret = sched_getparam(pid, &mut param as *mut sched_param);

        if ret != 0 {
            let err = *libc::__errno_location();
            match err {
                EPERM  => eprintln!("The requesting process does not have permission to obtain the scheduling parameters of the specified process"),
                ESRCH  => eprintln!("No process can be found corresponding to that specified by pid {}", pid),
                _ => eprintln!("unknow error: {}", err)
            }
            return;
        }
    }

    match cli.command {
        Commands::Read => {}
        Commands::Write(w) => {
            param.sched_priority = match w.prio {
                Some(p) => p,
                None => 0,
            };
            let sched_policy: i32 = w.sched.into();

            unsafe {
                #[cfg(any(target_env = "musl", target_os = "emscripten", target_env = "ohos"))]
                let ret = syscall(
                    SYS_sched_setscheduler,
                    pid,
                    sched_policy,
                    &param as *const sched_param,
                ) as i32;

                #[cfg(not(any(
                    target_env = "musl",
                    target_os = "emscripten",
                    target_env = "ohos"
                )))]
                let ret = sched_setscheduler(pid, sched_policy, &param as *const sched_param);

                if ret < 0 {
                    let err = *libc::__errno_location();
                    match err {
                        ENAVAIL => eprintln!(
                            "Invalid arguments: pid is negative or param is NULL
                              policy is not one of the recognized policies
                              param does not make sense for the specified policy"
                        ),
                        EPERM => {
                            eprintln!("The calling thread does not have appropriate privileges")
                        }
                        ESRCH => eprintln!("The thread whose ID is {} could not be found.", pid),
                        EINVAL => {
                            let min_prio = sched_get_priority_min(sched_policy);
                            let max_prio = sched_get_priority_max(sched_policy);

                            eprintln!(
                                "Invalid params for Sched Class {}, priority should between: {}/{}",
                                w.sched
                                    .to_possible_value()
                                    .map(|pv| String::from(pv.get_name()))
                                    .unwrap(),
                                min_prio,
                                max_prio
                            );
                        }
                        _ => eprintln!("unknow error: {}", err),
                    }
                    return;
                }
            }
        }
    }

    println!("sched    class: {}", get_scheduler(pid));
    println!("sched priority: {}", param.sched_priority);
}
