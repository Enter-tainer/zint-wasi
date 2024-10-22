macro_rules! make_runner {
    (
        Fn($($arg: ident: $arg_ty: ty),* $(,)?) -> Result<$returned: ty, $error: ty> $init: block
    ) => {
        {
            type Runner =
                Box<dyn Fn($($arg_ty),*) -> Result<$returned, $error> + Send + Sync + 'static>;
            static RUNNER: OnceLock<Runner> = OnceLock::new();
            RUNNER.get_or_init(|| $init)
        }
    };
    (
        fn($($arg: ident: $arg_ty: ty),* $(,)?) -> Result<$returned: ty, $error: ty> $init: block
    ) => {
        {
            type Runner = fn($($arg_ty),*) -> Result<$returned, $error>;
            static RUNNER: OnceLock<Runner> = OnceLock::new();
            RUNNER.get_or_init(|| $init)
        }
    };
}
