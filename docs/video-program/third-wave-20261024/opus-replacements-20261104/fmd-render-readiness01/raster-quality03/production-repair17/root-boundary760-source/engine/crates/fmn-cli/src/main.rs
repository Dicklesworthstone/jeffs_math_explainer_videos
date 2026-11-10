//! The `fmn` binary entry point.
#![forbid(unsafe_code)]

mod process_output;

use std::io::Read as _;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn quiet_human_doctor(args: &[std::ffi::OsString]) -> bool {
    let Some(args) = args
        .iter()
        .map(|arg| arg.to_str().map(str::to_owned))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    matches!(
        fmn_cli::parse_args(args),
        Ok(fmn_cli::Invocation::Doctor(fmn_cli::DoctorCommand {
            common: fmn_cli::CommonOptions {
                quiet: true,
                robot: false,
                ..
            },
            ..
        }))
    )
}

fn main() -> ExitCode {
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    let suppress_doctor_report = quiet_human_doctor(&args);
    let mut run_output = if fmn_cli::is_internal_studio_worker_os(&args) {
        fmn_cli::run_internal_studio_worker_os(&args)
    } else if fmn_cli::is_studio_invocation_os(&args) {
        let shutdown = Arc::new(AtomicBool::new(false));
        let watcher_shutdown = Arc::clone(&shutdown);
        let watcher = std::thread::Builder::new()
            .name("fmn-studio-stdin".to_owned())
            .spawn(move || {
                let mut input = std::io::stdin().lock();
                let mut discard = [0_u8; 1024];
                while input.read(&mut discard).is_ok_and(|read| read != 0) {}
                watcher_shutdown.store(true, Ordering::Release);
            });
        if let Err(error) = watcher {
            fmn_cli::RunOutput {
                code: fmn_cli::internal_exit_code(),
                stdout: String::new(),
                stderr: format!("fmn: cannot start Studio shutdown watcher: {error}\n"),
            }
        } else {
            let stdout = std::io::stdout();
            fmn_cli::run_studio_os(&args, &mut stdout.lock(), &shutdown)
        }
    } else {
        fmn_cli::run_os(args)
    };

    // `doctor` builds one complete capability snapshot before deciding whether
    // an optional requirement failed. In human quiet mode the snapshot is
    // non-error output, so suppress it while retaining any typed stderr error.
    // Robot records are never suppressed: they are the command's API.
    if suppress_doctor_report {
        run_output.stdout.clear();
    }

    let command_code = run_output.code;
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut stdout = stdout.lock();
    let mut stderr = stderr.lock();
    if process_output::publish(
        &mut stdout,
        &mut stderr,
        run_output.stdout.as_bytes(),
        run_output.stderr.as_bytes(),
    )
    .is_err()
    {
        return ExitCode::from(fmn_cli::internal_exit_code());
    }

    ExitCode::from(command_code)
}
