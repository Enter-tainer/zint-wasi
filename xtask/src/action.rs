use crate::*;
use std::io;
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
        run: Some(crate::actions::action_ensure_wasi_sdk)
    },
    StubPlugin: {
        arg: "", name: "stub wasi",
        require: [BuildPlugin],
        run: Some(crate::actions::action_stub_plugin)
    },
    EnsureWasmOpt: {
        arg: "", name: "prepare wasm-opt",
        require: [],
        run: Some(crate::actions::action_prepare_wasm_opt)
    },
    OptPlugin: {
        arg: "", name: "optimize wasm",
        require: [StubPlugin],
        run: Some(crate::actions::action_opt_plugin)
    },
    BuildPlugin: {
        arg: "build-plugin", name: "build plugin",
        require: [EnsureWasi],
        run: Some(crate::actions::action_build_plugin)
    },
    PackagePlugin: {
        arg: "package-plugin", name: "package plugin",
        require: [StubPlugin, OptPlugin],
        run: None
    },
    CompileManual: {
        arg: "build-manual", name: "compile manual",
        require: [PackagePlugin],
        run: Some(crate::actions::action_build_manual)
    },
    CompileExample: {
        arg: "", name: "compile example",
        require: [PackagePlugin],
        run: Some(crate::actions::action_build_example)
    },
    CopyLicense: {
        arg: "", name: "",
        require: [],
        run: Some(crate::actions::action_copy_license)
    },
    Package: {
        arg: "package", name: "package",
        require: [PackagePlugin, CompileManual, CompileExample, CopyLicense],
        run: None
    },
    All: { // alias for package
        arg: "all", name: "",
        require: [Package],
        run: None
    },
];
use Action::*;

impl Action {
    fn run_impl(self, executed: &mut HashSet<Self>, running: &mut Vec<Self>) -> ActionResult {
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
            action_try!(dep.run_impl(executed, running));
        }

        let result = if let Some(runner) = self.runner() {
            let has_name = if let Some(name) = self.name() {
                println!("[TASK]: {name}");
                true
            } else {
                false
            };
            let result = (runner)();
            match &result {
                ActionResult::Ok if has_name => println!("[OK]"),
                ActionResult::Skip { reason: None } if has_name => println!("[SKIPPED]"),
                ActionResult::Skip {
                    reason: Some(reason),
                } if has_name => {
                    println!("[SKIPPED]: {reason}");
                }
                ActionResult::Error(error) => {
                    println!("[ERROR]: {error}");
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
    pub fn run(self, executed: &mut HashSet<Self>) -> io::Result<()> {
        let mut running = Vec::with_capacity(8);
        if let ActionResult::Error(error) = self.run_impl(executed, &mut running) {
            Err(io::Error::new(io::ErrorKind::Other, error))
        } else {
            Ok(())
        }
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
