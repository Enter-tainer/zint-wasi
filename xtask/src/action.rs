use crate::*;
use std::{collections::HashSet, fmt::Display};

/*
OptPlugin, // wasm-opt typst-package/zint_typst_plugin.wasm -O3 --enable-bulk-memory -o typst-package/zint_typst_plugin.wasm
*/

// hidden for readability
include!("./action_macros.rs");

// - if `arg` is empty, action can't be ran from xtask command
// - if `name` is empty, action exectuion will be hidden
declare_actions![
    EnsureWasi: {
        arg: "", name: "prepare WASI SDK",
        require: [],
        run: Some(action_ensure_wasi_sdk)
    },
    EnsureWasiStub: {
        arg: "", name: "prepare wasi-stub",
        require: [],
        run: Some(action_ensure_wasi_stub)
    },
    StubPlugin: {
        arg: "", name: "stub wasi",
        require: [BuildPlugin, EnsureWasiStub],
        run: Some(action_stub_plugin)
    },
    EnsureWasmOpt: {
        arg: "", name: "prepare wasm-opt",
        require: [],
        run: Some(action_prepare_wasm_opt)
    },
    OptPlugin: {
        arg: "", name: "optimize wasm",
        require: [EnsureWasmOpt, StubPlugin],
        run: Some(action_opt_plugin)
    },
    BuildPlugin: {
        arg: "build-plugin", name: "build plugin",
        require: [EnsureWasi],
        run: Some(action_build_plugin)
    },
    PackagePlugin: {
        arg: "package-plugin", name: "package plugin",
        require: [StubPlugin, OptPlugin],
        run: None
    },
    CompileManual: {
        arg: "build-manual", name: "compile manual",
        require: [PackagePlugin],
        run: Some(action_build_manual)
    },
    CompileExample: {
        arg: "", name: "compile example",
        require: [PackagePlugin],
        run: Some(action_build_example)
    },
    CopyLicense: {
        arg: "", name: "",
        require: [],
        run: Some(action_copy_license)
    },
    EnsureCargoAbout: {
        arg: "", name: "",
        require: [],
        run: Some(action_ensure_cargo_about)
    },
    ThirdPartyLicense: {
        arg: "", name: "generate 3rd-party license list",
        require: [EnsureCargoAbout],
        run: Some(action_make_3rdparty_license_list)
    },
    Package: {
        arg: "package", name: "package",
        require: [PackagePlugin, CompileManual, CompileExample, CopyLicense, ThirdPartyLicense],
        run: None
    },
    InstallTypst: {
        arg: "", name: "",
        require: [],
        run: Some(action_install_typst)
    },
    RunCI: {
        arg: "ci", name: "",
        require: [PackagePlugin, InstallTypst, CompileManual, ThirdPartyLicense],
        run: None
    },
    SetVersion: {
        arg: "set-version", name: "set version",
        require: [],
        run: Some(action_set_version)
    },
    // Hidden so that its stdout is the version alone, for scripts to read.
    Version: {
        arg: "version", name: "",
        require: [],
        run: Some(action_print_version)
    },
    // Typst is installed first so that a machine without it still gets the
    // manual, and the bundle is what a release is cut from.
    Bundle: {
        arg: "bundle", name: "bundle package",
        require: [InstallTypst, Package],
        run: Some(action_bundle)
    },
    All: { // alias for package
        arg: "all", name: "",
        require: [Package],
        run: None
    },
];
use Action::*;

#[allow(clippy::derivable_impls)]
impl Default for Action {
    fn default() -> Self {
        All
    }
}

impl Action {
    fn run_impl(
        self,
        executed: &mut HashSet<Self>,
        running: &mut Vec<Self>,
        args: impl AsRef<[String]>,
    ) -> ActionResult {
        if running.contains(&self) {
            let names: Vec<_> = running
                .iter()
                .chain([&self])
                .map(|it| format!("{:?}", it))
                .collect();
            let names = names.join(">");
            unreachable!("action dependency cycle in path: {}", names)
        }

        if executed.contains(&self) {
            action_skip!("already executed");
        } else {
            running.push(self);
        }

        for dep in self.dependencies() {
            action_try!(dep.run_impl(executed, running, args.as_ref()));
        }

        let result = if let Some(runner) = self.runner() {
            let has_name = if let Some(name) = self.name() {
                info!("[TASK]: {}", name);
                true
            } else {
                false
            };
            let result = (runner)(args.as_ref());
            match &result {
                ActionResult::Ok if has_name => info!("[OK]"),
                ActionResult::Skip { reason: None } if has_name => info!("[SKIPPED]"),
                ActionResult::Skip {
                    reason: Some(reason),
                } if has_name => {
                    info!("[SKIPPED]: {}", reason);
                }
                ActionResult::Error(err) => {
                    error!(err);
                    std::process::exit(1);
                }
                _ => {}
            }
            result
        } else {
            ActionResult::Ok
        };

        executed.insert(self);
        running.pop();

        result
    }

    #[inline]
    pub fn run(self, args: impl IntoIterator<Item = String>) {
        let mut executed = HashSet::new();
        let mut running = Vec::with_capacity(8);
        let args: Vec<_> = args.into_iter().collect();
        let _ = self.run_impl(&mut executed, &mut running, &args);
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.name() {
            Some(name) => f.write_str(name),
            None => write!(f, "{:?}", self),
        }
    }
}

pub enum ActionResult {
    Ok,
    Skip { reason: Option<String> },
    Error(Box<dyn std::error::Error + Send + Sync + 'static>),
}
impl ActionResult {
    #[inline]
    pub fn error<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        ActionResult::Error(Box::new(error))
    }
}

pub mod macros {
    #[macro_export]
    macro_rules! action_ok {
        () => {
            return $crate::action::ActionResult::Ok;
        };
    }
    #[macro_export]
    macro_rules! action_skip {
        () => {
            return $crate::action::ActionResult::Skip {
                reason: None
            }
        };
        ($reason: literal) => {
            return $crate::action::ActionResult::Skip {
                reason: Some($reason.to_string())
            }
        };
        ($reason: literal, $($arg: expr),+) => {
            return $crate::action::ActionResult::Skip {
                reason: Some(format!($reason, $($arg),+))
            }
        };
    }
    #[macro_export]
    macro_rules! action_error {
        ($error: expr) => {
            return $crate::action::ActionResult::error($error)
        };
    }
    #[macro_export]
    macro_rules! action_expect {
        (cargo([$($args: expr),*])) => {{
            let status = match cargo([$($args),*]) {
                Ok(it) => it,
                Err(error) => $crate::action_error!(error),
            }.status();
            let status = match status {
                Ok(it) => it,
                Err(_) => panic!("can't run cargo"),
            };
            action_expect!(CommandError::from_exit(status))
        }};
        ($stmt: expr) => {{
            match $stmt {
                Ok(it) => it,
                Err(error) => $crate::action_error!(error),
            }
        }};
    }
    #[macro_export]
    macro_rules! action_expect_0 {
        (cargo([$($args: expr),*])) => {{
            $crate::action_expect!($crate::tools::CommandError::from_exit(
                $crate::action_expect!(cargo([$($args),*]))
            ).map_err(|err| err.program("cargo")))
        }};
        (cmd($name: literal, [$($args: expr),*])) => {{
            $crate::action_expect!($crate::tools::CommandError::from_exit(
                $crate::action_expect!(cmd($name, [$($args),*]))
            ).map_err(|err| err.program($name)))
        }};
        (cmd($program: literal as $name: literal, [$($args: expr),*])) => {{
            $crate::action_expect!($crate::tools::CommandError::from_exit(
                $crate::action_expect!(cmd($program, [$($args),*]))
            ).map_err(|err| err.program($name)))
        }};
        (cmd($name: literal, [$($args: expr),*])) => {{
            $crate::action_expect!($crate::tools::CommandError::from_exit(
                $crate::action_expect!(cmd($name, [$($args),*]))
            ).map_err(|err| err.program(stringify!($name))))
        }};
        (cmd($program: ident as $name: literal, [$($args: expr),*])) => {{
            $crate::action_expect!($crate::tools::CommandError::from_exit(
                $crate::action_expect!(cmd($program, [$($args),*]))
            ).map_err(|err| err.program($name)))
        }};
        ($stmt: expr) => {{
            $crate::action_expect!($crate::tools::CommandError::from_exit(
                $crate::action_expect!($stmt)
            ))
        }};
    }
    #[macro_export]
    macro_rules! action_try {
        ($stmt: expr) => {{
            if let $crate::action::ActionResult::Error(error) = $stmt {
                return $crate::action::ActionResult::Error(error);
            }
        }};
    }

    #[allow(unused_imports)]
    pub use crate::{
        action_error, action_expect, action_expect_0, action_ok, action_skip, action_try,
    };
}

#[path = "./action_impl.rs"]
mod action_impl;
pub use action_impl::*;

#[cfg(test)]
mod tests {
    use super::Action;
    use std::collections::HashSet;

    /// Declares every action, so the graph checks below cover all of them.
    ///
    /// The generated `match` keeps the list honest: an action that is added
    /// without being listed here does not compile.
    macro_rules! all_actions {
        ($($action:ident),+ $(,)?) => {
            const ALL: &[Action] = &[$(Action::$action),+];

            #[allow(dead_code)]
            fn every_action_is_listed(action: Action) {
                match action {
                    $(Action::$action => ()),+
                }
            }
        };
    }

    all_actions![
        EnsureWasi,
        EnsureWasiStub,
        StubPlugin,
        EnsureWasmOpt,
        OptPlugin,
        BuildPlugin,
        PackagePlugin,
        CompileManual,
        CompileExample,
        CopyLicense,
        EnsureCargoAbout,
        ThirdPartyLicense,
        Package,
        InstallTypst,
        RunCI,
        SetVersion,
        Version,
        Bundle,
        All,
    ];

    /// Everything reachable from `action`, which is what running it would do.
    fn reachable(action: Action) -> HashSet<Action> {
        let mut found = HashSet::new();
        let mut queue = vec![action];
        while let Some(current) = queue.pop() {
            for dependency in current.dependencies() {
                if found.insert(*dependency) {
                    queue.push(*dependency);
                }
            }
        }
        found
    }

    fn assert_acyclic(action: Action, done: &mut HashSet<Action>, path: &mut Vec<Action>) {
        for dependency in action.dependencies() {
            assert!(
                !path.contains(dependency),
                "dependency cycle: {}",
                path.iter()
                    .chain([dependency])
                    .map(|it| format!("{it:?}"))
                    .collect::<Vec<_>>()
                    .join(" > ")
            );
            if !done.insert(*dependency) {
                continue;
            }
            path.push(*dependency);
            assert_acyclic(*dependency, done, path);
            path.pop();
        }
    }

    #[test]
    fn the_tasks_the_readme_documents_can_be_asked_for() {
        for (argument, expected) in [
            ("build-plugin", Action::BuildPlugin),
            ("package-plugin", Action::PackagePlugin),
            ("build-manual", Action::CompileManual),
            ("package", Action::Package),
            ("ci", Action::RunCI),
            ("set-version", Action::SetVersion),
            ("version", Action::Version),
            ("bundle", Action::Bundle),
            ("all", Action::All),
        ] {
            assert_eq!(Action::parse_arg(argument), Ok(expected), "{argument}");
        }
    }

    /// Actions without an argument are steps of a larger task rather than tasks
    /// of their own, and an empty argument must not reach them.
    #[test]
    fn an_unknown_task_is_reported_with_the_argument_that_was_given() {
        for argument in ["", " ", "opt-plugin", "Package", "build_plugin"] {
            assert_eq!(
                Action::parse_arg(argument),
                Err(argument.to_string()),
                "{argument:?}"
            );
        }
    }

    #[test]
    fn running_xtask_without_a_task_runs_everything() {
        assert_eq!(Action::default(), Action::All);
    }

    /// Walking the dependencies is how an action runs, and `run_impl` gives up
    /// with `unreachable!` if it finds a cycle. This is what proves the declared
    /// graph has none.
    #[test]
    fn no_action_depends_on_itself_directly_or_indirectly() {
        for action in ALL {
            let mut done = HashSet::new();
            let mut path = vec![*action];
            assert_acyclic(*action, &mut done, &mut path);
        }
    }

    /// The workflow runs `cargo xtask ci` and then uploads the plugin and the
    /// manual, so those steps have to stay part of that task. The licence list
    /// is part of it too, so that a dependency bringing in a licence that
    /// `about.toml` does not accept fails the pull request rather than the
    /// release.
    #[test]
    fn the_ci_task_still_produces_the_plugin_the_manual_and_the_licence_list() {
        let reached = reachable(Action::RunCI);

        for required in [
            Action::EnsureWasi,
            Action::BuildPlugin,
            Action::EnsureWasiStub,
            Action::StubPlugin,
            Action::EnsureWasmOpt,
            Action::OptPlugin,
            Action::PackagePlugin,
            Action::InstallTypst,
            Action::CompileManual,
            Action::EnsureCargoAbout,
            Action::ThirdPartyLicense,
        ] {
            assert!(
                reached.contains(&required),
                "{required:?} is no longer part of the CI task"
            );
        }
    }

    /// `wasi-stub` is run as a plain command off PATH, so the step that puts
    /// it there has to stay part of stubbing. Without it the build fails on a
    /// missing tool on any host that has never installed one.
    #[test]
    fn stubbing_installs_the_tool_it_runs() {
        assert!(
            reachable(Action::StubPlugin).contains(&Action::EnsureWasiStub),
            "stubbing no longer ensures wasi-stub is installed"
        );
    }

    /// Packaging is what a release is cut from, so it has to keep collecting
    /// the licence files along with the plugin. `cargo about` has to be
    /// installed on the way, or the third-party page is generated from a
    /// command that cannot run.
    #[test]
    fn packaging_still_collects_the_licences() {
        let reached = reachable(Action::Package);

        assert!(reached.contains(&Action::CopyLicense));
        assert!(reached.contains(&Action::ThirdPartyLicense));
        assert!(reached.contains(&Action::EnsureCargoAbout));
        assert!(reached.contains(&Action::PackagePlugin));
    }

    /// The release workflow runs `cargo xtask bundle` and zips what it lays
    /// out, so the bundle has to be built from a complete package, with the
    /// manual compiled by a typst that was fetched if the runner has none.
    #[test]
    fn the_bundle_task_packages_everything_first() {
        let reached = reachable(Action::Bundle);

        assert!(reached.contains(&Action::Package));
        assert!(reached.contains(&Action::InstallTypst));
        assert!(reached.contains(&Action::CompileManual));
        assert!(reached.contains(&Action::ThirdPartyLicense));
    }

    /// The version tasks are for scripts; if they pulled in a build, reading
    /// the version would take minutes and could fail for unrelated reasons.
    #[test]
    fn the_version_tasks_build_nothing() {
        assert!(Action::Version.dependencies().is_empty());
        assert!(Action::SetVersion.dependencies().is_empty());
    }

    #[test]
    fn only_named_actions_are_announced_while_they_run() {
        assert_eq!(Action::BuildPlugin.name(), Some("build plugin"));
        assert_eq!(Action::OptPlugin.name(), Some("optimize wasm"));
        assert_eq!(Action::CopyLicense.name(), None);
        // Its stdout is read by scripts, so nothing else may end up there.
        assert_eq!(Action::Version.name(), None);
        assert_eq!(Action::All.name(), None);
    }

    #[test]
    fn an_action_without_a_name_is_displayed_by_its_variant() {
        assert_eq!(Action::BuildPlugin.to_string(), "build plugin");
        assert_eq!(Action::All.to_string(), "All");
    }

    /// The actions that only group other ones have nothing to run themselves.
    #[test]
    fn grouping_actions_have_no_runner() {
        assert!(Action::Package.runner().is_none());
        assert!(Action::PackagePlugin.runner().is_none());
        assert!(Action::RunCI.runner().is_none());
        assert!(Action::All.runner().is_none());

        assert!(Action::BuildPlugin.runner().is_some());
        assert!(Action::OptPlugin.runner().is_some());
    }
}
