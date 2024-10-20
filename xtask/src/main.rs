use action::Action;
use state::GlobalState;

mod action;
mod log;
mod state;
mod tools;
mod util;

fn main() {
    let mut args = std::env::args().skip(1);

    let action: Action = match args.next().map(|it| Action::parse_arg(&it)) {
        Some(Ok(it)) => it,
        Some(Err(unknown)) => {
            error!("unknown task: {}", unknown);
            std::process::exit(1)
        }
        None => {
            info!("no task specified, running 'all'");
            Action::default()
        }
    };

    let working_directory = state_path!(WORK_DIR);
    if working_directory.exists() {
        std::fs::create_dir_all(working_directory).expect("unable to create work directory");
    }
    action.run(args);

    let _ = GlobalState::save();
}
