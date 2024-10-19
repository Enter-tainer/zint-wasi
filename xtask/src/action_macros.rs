macro_rules! declare_actions {
    ($($action: ident: {arg: $arg: literal, name: $name: literal, require: [$($dep: ident),* $(,)?], run: $runner: expr}),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Action {
            $($action,)*
        }

        impl Action {
            /// Used for displaying current step when running commands
            ///
            /// If `None` is returned, then the action won't be displayed when it's run.
            pub fn name(self) -> Option<&'static str> {
                let parsed = [
                    $((Self::$action, $name),)*
                ];
                parsed.into_iter()
                    .filter(|(_, name)| !name.is_empty())
                    .find_map(|(action, name)| if action == self {
                        Some(name)
                    } else {
                        None
                    })
            }

            /// Tries parsing argument string into an `Action`.
            /// Returns the input argument on failiure.
            pub fn parse_arg(argument: impl AsRef<str>) -> Result<Self, String> {
                let parsed = [
                    $(($arg, Self::$action),)*
                ];
                parsed.into_iter()
                    .filter(|(cmd, _)| !cmd.is_empty())
                    .find_map(|(cmd, action)| if cmd == argument.as_ref() {
                        Some(action)
                    } else {
                        None
                    })
                    .ok_or_else(|| argument.as_ref().to_string())
            }

            /// Provides a list of dependencies for each action.
            pub fn dependencies(self) -> &'static [Self] {
                match self {
                    $($action => &[$(Self::$dep),*],)*
                }
            }

            /// Provides optionally executed logic by an action.
            pub fn runner(self) -> Option<fn(&[String]) -> ActionResult> {
                match self {
                    $(Self::$action => $runner,)*
                }
            }
        }
    };
}
