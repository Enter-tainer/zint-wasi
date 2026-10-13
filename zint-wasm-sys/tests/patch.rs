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
    let head = head.trim();
    // A `*` in front of the name belongs to the return type: `int *f(`
    // declares `f`, returning `int *`.
    let (_, name) = head.rsplit_once(|c: char| c.is_whitespace() || c == '*')?;
    Some(Signature {
        name: name.to_string(),
        returns: declared_type(head),
        parameters: parameters.split(',').map(declared_type).collect(),
    })
}

/// The type a declaration gives, with the name it declares taken off. zint
/// names every parameter, so a lone word is a type with nothing after it, as
/// in `f(void)`.
///
/// Input:  `struct zint_symbol* symbol`
/// Output: `struct zint_symbol *`
fn declared_type(declaration: &str) -> String {
    let declaration = declaration.trim();
    let without_name = match declaration.rsplit_once(|c: char| c.is_whitespace() || c == '*') {
        Some((_, name)) => &declaration[..declaration.len() - name.len()],
        None => declaration,
    };
    without_name
        .replace('*', " * ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether `line` mentions `name` followed by its argument list, as opposed to
/// a longer name that merely ends in it.
///
/// Input:  `INTERNAL int eps_plot(` with `ps_plot`
/// Output: `false`
fn mentions(line: &str, name: &str) -> bool {
    line.match_indices(&format!("{name}("))
        .any(|(at, _)| !line[..at].ends_with(|c: char| c.is_alphanumeric() || c == '_'))
}

/// The stubs `source` defines.
///
/// Every line at file scope that is not blank, a comment, a preprocessor
/// directive or a closing brace has to open a definition the parser can read,
/// so a stub written some other way fails here rather than going unchecked.
fn parse_stubs(source: &str) -> Vec<Signature> {
    let stubs: Vec<_> = source
        .lines()
        .map(str::trim_end)
        .filter(|line| {
            !(line.is_empty()
                || line.starts_with(char::is_whitespace)
                || line.starts_with('#')
                || line.starts_with("//")
                || *line == "}")
        })
        .map(|line| {
            parse_signature(line)
                .filter(|_| line.ends_with('{'))
                .unwrap_or_else(|| panic!("not a one-line function definition: {line}"))
        })
        .collect();
    // Without this the test below would pass by finding nothing to compare.
    assert!(!stubs.is_empty(), "no function definitions were found");
    stubs
}

/// Every one-line `INTERNAL` declaration and definition of a function in
/// `stubbed` that `source` holds, with the line it is on.
///
/// A line that names one of them but cannot be read, such as a declaration
/// wrapped over several lines, fails here rather than dropping out of the
/// comparison.
fn declarations_in(file: &str, source: &str, stubbed: &[&str]) -> Vec<(String, Signature)> {
    let mut found = Vec::new();
    for (index, line) in source.lines().enumerate() {
        if !line.starts_with("INTERNAL ") {
            continue;
        }
        let location = format!("{file}:{}", index + 1);
        match parse_signature(line) {
            Some(signature) if stubbed.contains(&signature.name.as_str()) => {
                found.push((location, signature));
            }
            Some(_) => {}
            None => {
                if let Some(name) = stubbed.iter().find(|name| mentions(line, name)) {
                    panic!("{location} declares `{name}` in a form this test cannot read: {line}");
                }
            }
        }
    }
    found
}

/// Every declaration and definition of a function in `stubbed` in zint's
/// backend, from the files the build compiles and the ones it leaves out alike.
///
/// Only the backend directory itself is read. zint's own tests below it
/// declare some of these functions for themselves, and not always the way the
/// library does: `tests/test_ps.c` gives the PostScript backend's entry point
/// two parameters, where the library gives it one.
fn zint_declarations(stubbed: &[&str]) -> Vec<(String, Signature)> {
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
        found.extend(declarations_in(
            &file,
            &String::from_utf8_lossy(&source),
            stubbed,
        ));
    }
    found
}

/// zint calls each of these functions through its own declaration of it, so
/// the stub has to take exactly the parameters that declaration passes.
#[test]
fn every_stub_has_the_signature_zint_calls_it_with() {
    let stubs = parse_stubs(include_str!("../patch/patch.c"));
    let names: Vec<&str> = stubs.iter().map(|it| it.name.as_str()).collect();
    let declarations = zint_declarations(&names);

    for stub in &stubs {
        let matching: Vec<_> = declarations
            .iter()
            .filter(|(_, it)| it.name == stub.name)
            .collect();
        assert!(
            !matching.is_empty(),
            "patch.c stubs `{}`, which zint no longer declares",
            stub.name
        );
        for (location, declaration) in matching {
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

/// A call such as `error_number = ps_plot(symbol);` ends in `;` as a
/// declaration does, and would otherwise read as one returning
/// `error_number =`.
#[test]
fn a_call_is_not_mistaken_for_a_declaration() {
    assert_eq!(
        parse_signature("            error_number = ps_plot(symbol);"),
        None
    );
}

/// Otherwise a stub returning `int` for a function returning `int *` would be
/// reported as stubbing a function zint does not declare, rather than as a
/// mismatch.
#[test]
fn a_star_before_the_name_belongs_to_the_return_type() {
    let signature =
        parse_signature("INTERNAL unsigned char *buffer_of(struct zint_symbol *symbol);")
            .expect("a declaration");

    assert_eq!(signature.name, "buffer_of");
    assert_eq!(signature.returns, "unsigned char *");
}

#[test]
#[should_panic(expected = "not a one-line function definition")]
fn a_stub_the_parser_cannot_read_is_refused() {
    parse_stubs("int ps_plot(struct zint_symbol *symbol)\n{\n    return 0;\n}\n");
}

#[test]
#[should_panic(expected = "in a form this test cannot read")]
fn a_wrapped_declaration_of_a_stubbed_function_is_refused() {
    declarations_in(
        "library.c",
        "INTERNAL int plot_raster(struct zint_symbol *symbol,\n    int rotate_angle, int file_type);\n",
        &["plot_raster"],
    );
}

/// The check above must not catch a function that only ends in a stubbed
/// name, or every wrapped declaration that happens to would fail the test.
#[test]
fn a_longer_name_ending_in_a_stubbed_one_is_not_its_declaration() {
    let found = declarations_in(
        "ps.c",
        "INTERNAL int eps_plot(struct zint_symbol *symbol,\n    int rotate_angle);\n",
        &["ps_plot"],
    );

    assert!(found.is_empty(), "{found:?}");
}
