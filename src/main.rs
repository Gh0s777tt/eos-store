//! E-OS Store — Application store client for E-OS — browse, verify and install signed packages.
//!
//! Status: skeleton. One Slint window (title + status line) over the shared
//! `eos-ui` Slint-on-Orbital backend, plus the headless model in [`model`].
//!
//! Exit codes follow the E-OS convention (CLAUDE.md §13.1):
//! `0` success, `1` a check found a defect, `2` the check could not run.

mod model;

slint::include_modules!();

/// Usage text; also the answer to `--help`.
const USAGE: &str = "\
eos-store — E-OS Store

USAGE:
    eos-store [OPTIONS]

OPTIONS:
    --selftest        run the headless model proof, print EOS-STORE-SELFTEST-OK, exit 0
    --check-backend   install the window backend and build the window without
                      entering the event loop, print EOS-STORE-BACKEND-OK, exit 0
    --version         print name and version
    --help            print this text

EXIT CODES:
    0  success
    1  a check found a defect
    2  the check could not run (built without the `host-backend` feature on a
       non-Redox host, so no windowing backend is linked in)
";

/// Install the host window backend (winit), when this build has one.
///
/// Three mutually exclusive definitions instead of `cfg!` branches inside one
/// body: the "no backend" case must be a plain `Err`, not a diverging block,
/// or `clippy -D warnings` trips over the `return` in tail position.
#[cfg(all(feature = "host-backend", not(target_os = "redox")))]
fn install_host_backend() -> Result<(), String> {
    let backend = i_slint_backend_winit::Backend::new()
        .map_err(|e| format!("winit backend unavailable: {e}"))?;
    slint::platform::set_platform(Box::new(backend))
        .map_err(|_| "a Slint platform was already installed".to_string())
}

/// Fail-closed default on a host: no backend was linked in, so say so instead
/// of letting the window constructor fail with "no default Slint platform".
#[cfg(all(not(feature = "host-backend"), not(target_os = "redox")))]
fn install_host_backend() -> Result<(), String> {
    Err(
        "built without the `host-backend` feature: no windowing backend is linked in \
         (rebuild with `--features host-backend` for a host window)"
            .to_string(),
    )
}

/// On Redox the platform is Orbital, installed by `eos_ui::init` below.
#[cfg(target_os = "redox")]
fn install_host_backend() -> Result<(), String> {
    Ok(())
}

/// Install the windowing platform Slint will draw on: the host winit backend
/// when there is one, then the shared E-OS backend + font bootstrap (a no-op
/// off Redox).
fn install_platform() -> Result<(), String> {
    install_host_backend()?;
    eos_ui::init(model::APP_TITLE);
    Ok(())
}

/// Build the window and wire the model into it. Does not run the event loop.
fn build_window() -> Result<MainWindow, String> {
    let catalog = model::Catalog::new();
    let win = MainWindow::new().map_err(|e| format!("cannot create the window: {e}"))?;
    win.set_app_title(model::APP_TITLE.into());
    win.set_status(catalog.status_line().into());
    Ok(win)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{USAGE}");
        return;
    }

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("{} {}", model::APP_NAME, model::APP_VERSION);
        return;
    }

    if args.iter().any(|a| a == "--selftest") {
        match model::selftest() {
            Ok(()) => {
                // Printed on both streams so the marker lands in the boot
                // serial log however the probe is wired.
                println!("{}", model::SELFTEST_OK);
                eprintln!("{}", model::SELFTEST_OK);
            }
            Err(err) => {
                println!("EOS-STORE-SELFTEST-FAIL: {err}");
                eprintln!("EOS-STORE-SELFTEST-FAIL: {err}");
                std::process::exit(1);
            }
        }
        return;
    }

    if args.iter().any(|a| a == "--check-backend") {
        if let Err(err) = install_platform() {
            eprintln!("EOS-STORE-BACKEND-UNAVAILABLE: {err}");
            // 2, not 1: the binary was built without a backend, so this run
            // could not measure anything — the toolbox is wrong, not the code.
            std::process::exit(2);
        }
        match build_window() {
            Ok(_win) => println!("EOS-STORE-BACKEND-OK"),
            Err(err) => {
                eprintln!("EOS-STORE-BACKEND-FAIL: {err}");
                std::process::exit(1);
            }
        }
        return;
    }

    if let Some(unknown) = args.iter().find(|a| a.starts_with('-')) {
        eprintln!("eos-store: unknown option {unknown}\n");
        eprint!("{USAGE}");
        std::process::exit(1);
    }

    if let Err(err) = install_platform() {
        eprintln!("eos-store: {err}");
        std::process::exit(2);
    }
    let win = match build_window() {
        Ok(win) => win,
        Err(err) => {
            eprintln!("eos-store: {err}");
            std::process::exit(1);
        }
    };
    if let Err(err) = win.run() {
        eprintln!("eos-store: event loop failed: {err}");
        std::process::exit(1);
    }
}
