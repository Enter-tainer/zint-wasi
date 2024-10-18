use std::collections::HashSet;

use action::Action;

mod action;
mod actions;
mod tools;
mod state;
pub mod log;

fn main() {
    let args = std::env::args().skip(1);

    let actions: Result<Vec<Action>, String> =
        args.into_iter().map(|it| Action::parse_arg(&it)).collect();
    let mut actions = match actions {
        Ok(it) => it,
        Err(unknown) => {
            error!("unknown action: {}", unknown);
            std::process::exit(1)
        }
    };

    if actions.is_empty() {
        info!("no actions specified, running 'all'");
        actions.push(Action::All);
    }

    let mut executed = HashSet::new();
    for action in actions {
        if let Err(error) = action.run(&mut executed) {
            error!(error);
        }
    }
}
