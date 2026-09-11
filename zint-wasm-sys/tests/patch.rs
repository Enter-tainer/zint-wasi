//! Checks the stubs in `patch/patch.c` against the zint functions they stand
//! in for.
//!
//! `build.rs` leaves zint's raster, PostScript and EMF backends out of the
//! build and links these stubs in their place. Nothing else compares the two:
//! a stub and zint's declaration of it sit in different translation units, so
//! no C compiler sees both, and a native linker does not look at signatures.
//! The WebAssembly linker does, but it answers a mismatched call by turning it
//! into a trap rather than by failing the build.

use std::path::Path;

/// What a caller and the function it calls have to agree on: the return type
/// and the parameter types. Parameter names are left out.
#[derive(Debug, PartialEq, Eq)]
struct Signature {
    name: String,
    returns: String,
    parameters: Vec<String>,
}

/// Reads a function declaration, or the first line of a definition, written
/// on one line at file scope, which is how zint and `patch.c` write them.
///
/// Input:  `INTERNAL int emf_plot(struct zint_symbol *symbol, int rotate_angle);`
/// Output: name `emf_plot`, returns `int`, parameters `["struct zint_symbol *", "int"]`
fn parse_signature(line: &str) -> Option<Signature> {
    // A call ends in `;` as well, but never at file scope.
    if line.starts_with(char::is_whitespace) {
        return None;
    }
    let line = line.strip_prefix("INTERNAL ").unwrap_or(line);
    let (head, rest) = line.split_once('(')?;
    let (parameters, tail) = rest.split_once(')')?;
    if !matches!(tail.trim(), ";" | "{") {
        return None;
    }
    let (returns, name) = head.trim().rsplit_once(' ')?;
    Some(Signature {
        name: name.to_string(),
        returns: returns.trim().to_string(),
        parameters: parameters.split(',').map(parameter_type).collect(),
    })
}

/// The type a parameter declares, without its name. zint names every
/// parameter, so a lone word is a type with nothing after it, as in `f(void)`.
///
/// Input:  `struct zint_symbol* symbol`
/// Output: `struct zint_symbol *`
fn parameter_type(parameter: &str) -> String {
    let parameter = parameter.trim();
    let without_name = match parameter.rsplit_once(|c: char| c.is_whitespace() || c == '*') {
        Some((_, name)) => &parameter[..parameter.len() - name.len()],
        None => parameter,
    };
    without_name
        .replace('*', " * ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The stubs `patch.c` defines.
fn stubs() -> Vec<Signature> {
    let stubs: Vec<_> = include_str!("../patch/patch.c")
        .lines()
        .filter(|line| line.trim_end().ends_with('{'))
        .filter_map(parse_signature)
        .collect();
    // Without this the test below would pass by finding nothing to compare.
    assert!(
        !stubs.is_empty(),
        "no function definitions were found in patch.c"
    );
    stubs
}

/// Every declaration and definition of `name` in zint's backend, with the
/// file and line it is on, from the files the build compiles and the ones it
/// leaves out alike.
///
/// Only the backend directory itself is read. zint's own tests below it
/// declare some of these functions for themselves, and not always the way the
/// library does: `tests/test_ps.c` still gives `ps_plot` two parameters.
fn zint_declarations(name: &str) -> Vec<(String, Signature)> {
    let backend = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("zint")
        .join("backend");
    let mut found = Vec::new();
    for entry in std::fs::read_dir(&backend).expect("the zint submodule is checked out") {
        let path = entry.expect("the backend directory can be listed").path();
        if !matches!(path.extension().and_then(|it| it.to_str()), Some("c" | "h")) {
            continue;
        }
        let file = path.file_name().unwrap().to_string_lossy().into_owned();
        let source = std::fs::read(&path).expect("a backend source can be read");
        for (index, line) in String::from_utf8_lossy(&source).lines().enumerate() {
            if !line.starts_with("INTERNAL ") {
                continue;
            }
            if let Some(signature) = parse_signature(line).filter(|it| it.name == name) {
                found.push((format!("{file}:{}", index + 1), signature));
            }
        }
    }
    found
}

/// zint calls each of these functions through its own declaration of it, so
/// the stub has to take exactly the parameters that declaration passes.
#[test]
fn every_stub_has_the_signature_zint_calls_it_with() {
    for stub in stubs() {
        let declarations = zint_declarations(&stub.name);
        assert!(
            !declarations.is_empty(),
            "patch.c stubs `{}`, which zint no longer declares",
            stub.name
        );
        for (location, declaration) in declarations {
            assert_eq!(
                stub, declaration,
                "the stub in patch.c does not match {location}"
            );
        }
    }
}

#[test]
fn a_signature_reads_the_same_from_a_declaration_and_a_definition() {
    for line in [
        "INTERNAL int emf_plot(struct zint_symbol *symbol, int rotate_angle);",
        "int emf_plot(struct zint_symbol* symbol, int angle) {",
        // A CRLF checkout, read without `lines()` to strip the `\r`.
        "INTERNAL int emf_plot(struct zint_symbol *symbol, int rotate_angle) {\r",
    ] {
        assert_eq!(
            parse_signature(line),
            Some(Signature {
                name: "emf_plot".to_string(),
                returns: "int".to_string(),
                parameters: vec!["struct zint_symbol *".to_string(), "int".to_string()],
            }),
            "{line}"
        );
    }
}

/// `vector.c` calls `ps_plot` as `error_number = ps_plot(symbol);`, which
/// would otherwise read as a declaration returning `error_number =`.
#[test]
fn a_call_is_not_mistaken_for_a_declaration() {
    assert_eq!(
        parse_signature("            error_number = ps_plot(symbol);"),
        None
    );
}
