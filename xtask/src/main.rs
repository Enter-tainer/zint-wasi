use action::Action;
use state::State;

mod action;
mod actions;
mod tools;
mod state;
mod log;

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
        },
    };

    action.run(args);

    let _ = State::global_read().save(state::STATE_PATH);
}
