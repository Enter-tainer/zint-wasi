//! Golden file tests over the rendered SVG.
//!
//! There is one file per case under `tests/golden`: one for every symbology,
//! and one for every option that changes what gets drawn. The point is not that
//! the current output is correct, it is that a change to it is visible. An
//! upgrade of the vendored libzint, or a change to how options are passed to
//! it, shows up as a reviewable diff instead of as a silently different barcode.
//!
//! After an intended change, rewrite the files with
//!
//! ```sh
//! UPDATE_GOLDEN=1 cargo test -p zint-wasm-rs --test golden
//! ```
//!
//! and read the diff before committing it.

use std::{collections::BTreeSet, fs, path::PathBuf};

use zint_wasm_rs::{
    options::{
        color::Color,
        input_mode::InputMode,
        option3::{DataMatrixOption, Option3, QRMask, QRMatrixOption},
        output_options::OutputOptions,
        symbology::Symbology,
        Options,
    },
    symbol::Symbol,
};

const GOLDEN_ROOT: &str = concat![env!("CARGO_MANIFEST_DIR"), "/tests/golden"];
const UPDATE_VARIABLE: &str = "UPDATE_GOLDEN";

/// One barcode to render, and the name of the file holding what it should look
/// like.
struct Case {
    /// File stem inside the case's folder; also how a failure is reported.
    name: &'static str,
    data: &'static str,
    rotate: i32,
    options: Options,
}

impl Case {
    fn new(name: &'static str, symbology: Symbology, data: &'static str) -> Self {
        Self {
            name,
            data,
            rotate: 0,
            options: Options::with_symbology(symbology),
        }
    }

    /// The linear component of a composite symbol, or the structured data
    /// MaxiCode carries alongside its message.
    fn primary(mut self, primary: &str) -> Self {
        self.options.primary = Some(primary.to_string());
        self
    }

    fn option_1(mut self, value: i32) -> Self {
        self.options.option_1 = Some(value);
        self
    }

    fn option_2(mut self, value: i32) -> Self {
        self.options.option_2 = Some(value);
        self
    }

    fn option_3(mut self, value: Option3) -> Self {
        self.options.option_3 = Some(value);
        self
    }

    fn input_mode(mut self, value: InputMode) -> Self {
        self.options.input_mode = Some(value);
        self
    }

    fn rotated(mut self, degrees: i32) -> Self {
        self.rotate = degrees;
        self
    }

    /// For the options that only one case sets, so that they do not each need a
    /// builder method of their own.
    fn with(mut self, set: impl FnOnce(&mut Options)) -> Self {
        set(&mut self.options);
        self
    }
}

/// Renders a case, and either compares it against its golden file or rewrites
/// that file, depending on the `UPDATE_GOLDEN` environment variable.
///
/// Returns the failure to report, if there is one, so that a run can show every
/// case that drifted rather than only the first.
fn check(folder: &str, case: &Case) -> Result<(), String> {
    let rendered = Symbol::new(&case.options)
        .encode_svg(case.data, 0, case.rotate)
        .map_err(|error| format!("{folder}/{}: {error}", case.name))?;

    let path = golden_path(folder, case.name);
    if std::env::var_os(UPDATE_VARIABLE).is_some() {
        let parent = path.parent().expect("golden files live in a folder");
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
        return fs::write(&path, rendered).map_err(|error| format!("{}: {error}", path.display()));
    }

    let expected = fs::read_to_string(&path).map_err(|error| {
        format!(
            "{folder}/{}: {error}; run with {UPDATE_VARIABLE}=1 to write it",
            case.name
        )
    })?;

    // Git may have handed the file over with the platform's line endings, and
    // the renderer always produces Unix ones.
    let expected = expected.replace("\r\n", "\n");
    if expected == rendered {
        return Ok(());
    }

    Err(format!(
        "{folder}/{}: {}",
        case.name,
        first_difference(&expected, &rendered)
    ))
}

/// Describes where two renderings start to differ, so that a failing run says
/// what moved rather than only that something did.
///
/// Input:  a golden and a rendering whose third line is the `<svg>` element
/// Output:
///
/// ```text
/// line 3 differs
///   expected: <svg width="224" height="117" ...
///      found: <svg width="226" height="118" ...
/// ```
fn first_difference(expected: &str, actual: &str) -> String {
    for (number, (expected, actual)) in expected.lines().zip(actual.lines()).enumerate() {
        if expected != actual {
            return format!(
                "line {} differs\n  expected: {}\n     found: {}",
                number + 1,
                truncated(expected),
                truncated(actual)
            );
        }
    }

    format!(
        "expected {} lines, found {}",
        expected.lines().count(),
        actual.lines().count()
    )
}

fn truncated(line: &str) -> String {
    const LIMIT: usize = 120;
    match line.char_indices().nth(LIMIT) {
        Some((cut, _)) => format!("{}...", &line[..cut]),
        None => line.to_string(),
    }
}

fn golden_path(folder: &str, name: &str) -> PathBuf {
    PathBuf::from(GOLDEN_ROOT)
        .join(folder)
        .join(format!("{name}.svg"))
}

fn check_all(folder: &str, cases: &[Case]) {
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|case| check(folder, case).err())
        .collect();

    assert!(
        failures.is_empty(),
        "{} of {} golden files did not match:\n\n{}\n\nRe-run with {UPDATE_VARIABLE}=1 once the \
         change is understood and intended.",
        failures.len(),
        cases.len(),
        failures.join("\n\n")
    );
}

/// Every option that changes the drawing, one case each, so that a change in
/// how an option is passed to zint cannot pass unnoticed.
fn option_cases() -> Vec<Case> {
    const DATA: &str = "A12345B";

    vec![
        Case::new("scale-half", Symbology::Code128, DATA).with(|it| it.scale = Some(0.5)),
        Case::new("scale-double", Symbology::Code128, DATA).with(|it| it.scale = Some(2.0)),
        Case::new("height", Symbology::Code128, DATA).with(|it| it.height = Some(20.0)),
        Case::new("whitespace", Symbology::Code128, DATA).with(|it| {
            it.whitespace_width = Some(5);
            it.whitespace_height = Some(3);
        }),
        Case::new("border-bind", Symbology::Code128, DATA).with(|it| {
            it.border_width = Some(2);
            it.output_options = Some(OutputOptions::BARCODE_BIND);
        }),
        Case::new("border-box", Symbology::Code128, DATA).with(|it| {
            it.border_width = Some(2);
            it.output_options = Some(OutputOptions::BARCODE_BOX);
        }),
        Case::new("bind-top", Symbology::Code128, DATA).with(|it| {
            it.border_width = Some(2);
            it.output_options = Some(OutputOptions::BARCODE_BIND_TOP);
        }),
        // The corporate colours of a shipping label: an opaque background is
        // drawn as a rectangle, which a transparent one is not.
        Case::new("colors-opaque", Symbology::Code128, DATA).with(|it| {
            it.fg_color = Some(hex("#1F4788"));
            it.bg_color = Some(hex("#FFD700"));
        }),
        Case::new("colors-translucent", Symbology::Code128, DATA).with(|it| {
            it.fg_color = Some(hex("#1F478880"));
            it.bg_color = Some(hex("#FFD70040"));
        }),
        Case::new("no-hrt", Symbology::Code128, DATA).with(|it| it.show_hrt = Some(false)),
        Case::new("small-text", Symbology::Code128, DATA)
            .with(|it| it.output_options = Some(OutputOptions::SMALL_TEXT)),
        Case::new("bold-text", Symbology::Code128, DATA)
            .with(|it| it.output_options = Some(OutputOptions::BOLD_TEXT)),
        Case::new("text-gap", Symbology::Code128, DATA).with(|it| it.text_gap = Some(3.0)),
        Case::new("reader-init", Symbology::Code128, DATA)
            .with(|it| it.output_options = Some(OutputOptions::READER_INIT)),
        Case::new("compliant-height", Symbology::Code39, DATA)
            .with(|it| it.output_options = Some(OutputOptions::COMPLIANT_HEIGHT)),
        // Quiet zones and guard bars only mean something for the retail
        // symbologies, so they are exercised on an EAN-13.
        Case::new("quiet-zones", Symbology::EANXChk, "6975004310001")
            .with(|it| it.output_options = Some(OutputOptions::BARCODE_QUIET_ZONES)),
        Case::new("no-quiet-zones", Symbology::EANXChk, "6975004310001")
            .with(|it| it.output_options = Some(OutputOptions::BARCODE_NO_QUIET_ZONES)),
        Case::new("guard-whitespace", Symbology::EANXChk, "6975004310001")
            .with(|it| it.output_options = Some(OutputOptions::EAN_UPC_GUARD_WHITESPACE)),
        Case::new("guard-descent", Symbology::EANXChk, "6975004310001")
            .with(|it| it.guard_descent = Some(8.0)),
        // Dots are only drawn for matrix symbologies, and their size is only
        // read in that mode.
        Case::new("dotty", Symbology::DataMatrix, DATA).with(|it| {
            it.output_options = Some(OutputOptions::BARCODE_DOTTY_MODE);
            it.dot_size = Some(0.6);
        }),
        Case::new("eci", Symbology::QRCode, "Ünicode").with(|it| it.eci = Some(26)),
        Case::new("input-mode-escape", Symbology::Code128, "A12\\x34B")
            .input_mode(InputMode::ESCAPE),
        Case::new(
            "input-mode-gs1",
            Symbology::GS1128,
            "[01]09501101020917[10]AB-123",
        )
        .input_mode(InputMode::GS1),
        Case::new(
            "input-mode-gs1-parentheses",
            Symbology::GS1128,
            "(01)09501101020917(10)AB-123",
        )
        .input_mode(InputMode::GS1 | InputMode::GS1_PARENTHESES),
        Case::new(
            "gs1-gs-separator",
            Symbology::DataMatrix,
            "[01]09501101020917[10]AB-123",
        )
        .input_mode(InputMode::GS1)
        .with(|it| it.output_options = Some(OutputOptions::GS1_GS_SEPARATOR)),
        // Error correction level and version: the two symbol specific options
        // most documents reach for.
        Case::new(
            "option-1-error-correction",
            Symbology::QRCode,
            "https://example.com",
        )
        .option_1(4),
        Case::new("option-2-version", Symbology::QRCode, "https://example.com").option_2(5),
        Case::new("option-3-qr-mask", Symbology::QRCode, "https://example.com")
            .option_3(Option3::from(QRMatrixOption::from(QRMask::Mask3))),
        Case::new("option-3-data-matrix-square", Symbology::DataMatrix, DATA)
            .option_3(Option3::from(DataMatrixOption::Square)),
        Case::new("rotate-90", Symbology::Code128, DATA).rotated(90),
        Case::new("rotate-180", Symbology::Code128, DATA).rotated(180),
        Case::new("rotate-270", Symbology::Code128, DATA).rotated(270),
    ]
}

/// Parses a colour, panicking on a typo in a case above.
fn hex(value: &str) -> Color {
    use std::str::FromStr;
    Color::from_str(value).unwrap_or_else(|error| panic!("{value} is not a color: {error}"))
}

#[test]
fn every_option_matches_its_golden_file() {
    check_all("option", &option_cases());
}

#[test]
fn every_symbology_matches_its_golden_file() {
    check_all("symbology", &symbology_cases());
}

/// A renamed or removed case would otherwise leave its file behind, and a
/// leftover file looks exactly like a covered case.
#[test]
fn no_golden_file_is_left_behind() {
    let expected: BTreeSet<PathBuf> =
        [("option", option_cases()), ("symbology", symbology_cases())]
            .iter()
            .flat_map(|(folder, cases)| {
                cases
                    .iter()
                    .map(move |case| golden_path(folder, case.name))
                    .collect::<Vec<_>>()
            })
            .collect();

    let mut orphans = Vec::new();
    for folder in ["option", "symbology"] {
        let directory = PathBuf::from(GOLDEN_ROOT).join(folder);
        // The folders do not exist yet the first time the files are written.
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries {
            let path = entry.expect("readable directory entry").path();
            if !expected.contains(&path) {
                orphans.push(path.display().to_string());
            }
        }
    }

    assert!(
        orphans.is_empty(),
        "these golden files belong to no case any more:\n{}",
        orphans.join("\n")
    );
}

/// One barcode per symbology, so that a change in what any of them draws is
/// visible. The payloads are taken from libzint's own test vectors and
/// documentation, and are meant to be what that symbology carries in practice.
fn symbology_cases() -> Vec<Case> {
    vec![
        // Telecoms equipment identifier; digits only, two mod-11 check digits
        // appended by zint.
        Case::new("Code11", Symbology::Code11, "9212320967"),
        // Photo-lab job number; digits only, well inside the 112 digit limit.
        Case::new("C25Standard", Symbology::C25Standard, "9212320967"),
        // DX film cartridge code; six digits, an even count so no leading zero is
        // added.
        Case::new("C25Inter", Symbology::C25Inter, "602003"),
        // Ten-digit IATA baggage tag licence plate; digits only, limit is 80.
        Case::new("C25IATA", Symbology::C25IATA, "0220123456"),
        // Internal despatch note number; digits only, limit is 113.
        Case::new("C25Logic", Symbology::C25Logic, "5401962"),
        // Production batch number; digits only, limit is 79.
        Case::new("C25Ind", Symbology::C25Ind, "70412398"),
        // Asset tag; letters, digits and dash are all in the Code 39 character
        // set.
        Case::new("Code39", Symbology::Code39, "ASSET-0042189"),
        // Mixed-case item description; Extended Code 39 covers the full 7-bit
        // ASCII set.
        Case::new("ExCode39", Symbology::ExCode39, "123.45$@fd"),
        // Retail EAN-13 article number that already carries its correct check
        // digit.
        Case::new("EANX", Symbology::EANX, "5024425377399"),
        // EAN-13 supplied with its check digit, which zint validates rather than
        // calculates.
        Case::new("EANXChk", Symbology::EANXChk, "9501101531000"),
        // GTIN plus net weight and best-before date; GS1 input mode with square-
        // bracket AI delimiters.
        Case::new("GS1128", Symbology::GS1128, "[01]98898765432106[3202]012345[15]991231")
            .input_mode(InputMode::GS1),
        // Blood-bank donation number; starts and ends with a Codabar guard letter
        // as required.
        Case::new("Codabar", Symbology::Codabar, "A37859B"),
        // Parcel tracking number; Code 128 encodes the whole ASCII set, so any
        // printable payload is valid.
        Case::new("Code128", Symbology::Code128, "1Z999AA10123456784"),
        // Deutsche Post Leitcode routing number; exactly 13 digits, check digit
        // added by zint.
        Case::new("DPLEIT", Symbology::DPLEIT, "5082300702800"),
        // Deutsche Post Identcode consignment number; exactly 11 digits, check
        // digit added by zint.
        Case::new("DPIDENT", Symbology::DPIDENT, "39601313414"),
        // Worked example from the Code 16K standard, mixing Code 128 set B and set
        // C data; the two mod-107 check digits are added by Zint.
        Case::new("Code16k", Symbology::Code16k, "ab0123456789"),
        // Worked example from the Code 49 standard; 24 characters of 7-bit ASCII,
        // well inside the 49-character limit, rows chosen automatically.
        Case::new("Code49", Symbology::Code49, "MULTIPLE ROWS IN CODE 49"),
        // Warehouse inventory reference; every character is in the Code 93 base
        // set and the limit is 123.
        Case::new("Code93", Symbology::Code93, "INV-2024-000517"),
        // Print-shop page sequence mark; digits only, up to 128 of them.
        Case::new("Flat", Symbology::Flat, "1304056"),
        // 13-digit GS1 item code from the General Specifications figure; the check
        // digit and the HRT-only "(01)" are added by Zint.
        Case::new("DBarOmn", Symbology::DBarOmn, "0950110153001"),
        // 13-digit item code starting with 1, which DataBar Limited requires
        // (values below 2000000000000 only).
        Case::new("DBarLtd", Symbology::DBarLtd, "1501234567890"),
        // GS1's own DataBar Expanded example: GTIN, expiry date 2014-07-04 and
        // batch AB-123; AIs use square brackets so GS1 mode is set.
        Case::new("DBarExp", Symbology::DBarExp, "[01]09501101530003[17]140704[10]AB-123")
            .input_mode(InputMode::GS1),
        // UK library accession number; plain ASCII, well inside the 69 character
        // limit.
        Case::new("Telepen", Symbology::Telepen, "L0012345678"),
        // Eleven-digit US retail article number; zint calculates the twelfth check
        // digit.
        Case::new("UPCA", Symbology::UPCA, "61414123440"),
        // Full twelve-digit UPC-A including the check digit, which zint validates.
        Case::new("UPCAChk", Symbology::UPCAChk, "614141000302"),
        // Zero-compressed UPC-E article number for a small package; zint
        // calculates the check digit.
        Case::new("UPCE", Symbology::UPCE, "0012345"),
        // UPC-E supplied with its check digit, which zint validates rather than
        // calculates.
        Case::new("UPCEChk", Symbology::UPCEChk, "00783491"),
        // US ZIP+4 plus 2-digit delivery point (the 11-digit PostNet12 form); Zint
        // appends the mod-10 check digit.
        Case::new("Postnet", Symbology::Postnet, "12345678901"),
        // Warehouse shelf-label number; digits only, and no check digit is added
        // by default.
        Case::new("MSIPlessey", Symbology::MSIPlessey, "7591046238"),
        // FIM C, the single-letter mark USPS prints on courtesy-reply mail that
        // already carries a POSTNET barcode.
        Case::new("FIM", Symbology::FIM, "C"),
        // US DoD part marking from the MIL-STD-1189 worked example; 11 Code 39
        // characters, limit is 30.
        Case::new("Logmars", Symbology::Logmars, "12345/ABCDE"),
        // Laetus pharmacode for a medicine pack; the value sits inside the
        // permitted 3 to 131070 range.
        Case::new("Pharma", Symbology::Pharma, "130170"),
        // German Pharmazentralnummer from the IFA check-digit worked example; zint
        // appends the mod-11 check digit.
        Case::new("PZN", Symbology::PZN, "2758089"),
        // Two-track Laetus pharmacode; the value sits inside the permitted 4 to
        // 64570080 range.
        Case::new("PharmaTwo", Symbology::PharmaTwo, "29876543"),
        // Brazilian CEP 36400-000 as the required eight digits; Correios check
        // digit is added by Zint.
        Case::new("CEPNet", Symbology::CEPNet, "36400000"),
        // Courier parcel-label data string (consignment number plus label format
        // and route fields); plain ASCII that auto-sizes with the default error
        // correction level.
        Case::new("PDF417", Symbology::PDF417, "90044030118100801265*D_2D+1.02+31351440315981"),
        // The ISO/IEC 15438:2015 Figure G.1 message including its trailing line
        // feed, shown there in the compact (truncated) variant.
        Case::new("PDF417Comp", Symbology::PDF417Comp, "PDF417 APK\n"),
        // Parcel label from ISO/IEC 16023 Annex B whose alphanumeric postcode
        // B1050 plus country 056 and service class 999 auto-select Mode 3, with
        // the delivery address as the secondary message.
        Case::new("MaxiCode", Symbology::MaxiCode, "COMMISSION FOR EUROPEAN NORMALIZATION, RUE DE STASSART 36, B-1050 BRUXELLES")
            .primary("B1050056999"),
        // GS1 element string pairing a GTIN with an AI (8200) extended-packaging
        // product URL, from GS1 General Specifications Figure 5.7.3-1; GS1 mode
        // needs the [] AI delimiters.
        Case::new("QRCode", Symbology::QRCode, "[01]00857674002010[8200]http://www.gs1.org/")
            .input_mode(InputMode::GS1),
        // Batch reference for a reader that cannot handle Code Set C; alphanumeric
        // so no numeric compaction is lost.
        Case::new("Code128AB", Symbology::Code128AB, "BATCH-2024-0451"),
        // 8-digit Australia Post Delivery Point ID; Zint supplies FCC 11 and the
        // Reed-Solomon data.
        Case::new("AusPost", Symbology::AusPost, "96184209"),
        // 8-digit Delivery Point ID for an Australia Post Reply Paid item (FCC 45
        // added by Zint).
        Case::new("AusReply", Symbology::AusReply, "12345678"),
        // 8-digit Delivery Point ID for an Australia Post Routing barcode (FCC 87
        // added by Zint).
        Case::new("AusRoute", Symbology::AusRoute, "34567890"),
        // 8-digit Delivery Point ID for an Australia Post Redirection barcode (FCC
        // 92 added by Zint).
        Case::new("AusRedirect", Symbology::AusRedirect, "98765432"),
        // ISBN-13 whose check digit verifies, encoded as a Bookland EAN-13.
        Case::new("ISBNX", Symbology::ISBNX, "9789295055124"),
        // UK postcode W1J 0TR followed by house number 01, the usual RM4SCC
        // layout; check digit added by Zint.
        Case::new("RM4SCC", Symbology::RM4SCC, "W1J0TR01"),
        // GS1 Digital Link URI carrying a GTIN, the GS1 General Specifications
        // Figure 2.1.13.1 example, encoded as plain (non-GS1) data.
        Case::new("DataMatrix", Symbology::DataMatrix, "https://example.com/01/09506000134369"),
        // GTIN of a trade unit as 13 raw digits; zint prepends AI (01) and appends
        // the check digit.
        Case::new("EAN14", Symbology::EAN14, "9889876543210"),
        // North American vehicle identification number; 17 characters and the
        // position-9 check character verifies.
        Case::new("VIN", Symbology::VIN, "1FTCR10UXTPA78180"),
        // Example from the Codablock-F specification; with default options Zint
        // picks the row and column count itself.
        Case::new("CodablockF", Symbology::CodablockF, "CODABLOCK F Symbology"),
        // Seventeen-digit SSCC for a pallet; zint prepends AI (00) and adds both
        // check digits.
        Case::new("NVE18", Symbology::NVE18, "37612345000001003"),
        // Japan Post address code: postcode 154-0023 followed by the
        // block/building part; mod-19 check digit added by Zint.
        Case::new("JapanPost", Symbology::JapanPost, "15400233-16-4-205"),
        // Six-digit Korean postal code, the maximum length this symbology accepts;
        // the mod-10 check digit is added by Zint.
        Case::new("KoreaPost", Symbology::KoreaPost, "923457"),
        // Same 13-digit item code input as DataBar Truncated, taken from GS1's
        // DataBar page.
        Case::new("DBarStk", Symbology::DBarStk, "0950110153000"),
        // 13-digit item code from the ISO 24724 Stacked Omnidirectional figure;
        // check digit added by Zint.
        Case::new("DBarOmnStk", Symbology::DBarOmnStk, "0003456789012"),
        // ISO 24724 Expanded Stacked figure: GTIN with its own check digit, net
        // weight and best-before date; AIs use square brackets so GS1 mode is set.
        Case::new("DBarExpStk", Symbology::DBarExpStk, "[01]98898765432106[3202]012345[15]991231")
            .input_mode(InputMode::GS1),
        // 13-digit USPS PLANET routing code (the Planet14 standard length); Zint
        // appends the mod-10 check digit.
        Case::new("Planet", Symbology::Planet, "4012345235636"),
        // Eight-digit item number, the manual's MicroPDF417 example, small enough
        // for an auto-selected single-column symbol.
        Case::new("MicroPDF417", Symbology::MicroPDF417, "12345678"),
        // USPS Intelligent Mail: 20-digit tracking code, dash, then an 11-digit
        // delivery point ZIP.
        Case::new("USPSIMail", Symbology::USPSIMail, "01234567094987654321-01234567891"),
        // Library book accession number; digits and A-F only, with a hidden CRC
        // added by zint.
        Case::new("Plessey", Symbology::Plessey, "0037861"),
        // UK library barcode compressed as digit pairs; even length so no padding
        // zero is inserted.
        Case::new("TelepenNum", Symbology::TelepenNum, "30012345678901"),
        // GS1 shipping-container case code; exactly 13 digits, check digit added
        // by zint.
        Case::new("ITF14", Symbology::ITF14, "0950110153000"),
        // PostNL KIX string for postcode 2500 GG plus house-number data, exactly
        // the required 11 characters and no check digit.
        Case::new("KIX", Symbology::KIX, "2500GG30250"),
        // GS1 element string (GTIN, expiry, batch, ship-to GLN) of the kind used
        // on healthcare and logistics labels; GS1 mode needs the [] AI delimiters.
        Case::new("Aztec", Symbology::Aztec, "[01]03453120000011[17]120508[10]ABCD1234[410]9501101020917")
            .input_mode(InputMode::GS1),
        // Externally generated 4-state pattern that draws the same bars as RM4SCC
        // "W1J0TR01"; the manual's --height/--vers are cosmetic, defaults encode
        // fine.
        Case::new("DAFT", Symbology::DAFT, "AAFDTTDAFADTFTTFFFDATFTADTTFFTDAFAFDTF"),
        // 27-character DPD parcel label: destination post code 0081827, tracking
        // 09980000020028, service 101, country 276 (Germany); Zint prefixes the
        // "%" identification tag.
        Case::new("DPD", Symbology::DPD, "008182709980000020028101276"),
        // Eight-digit part number, the ISO/IEC 18004 Figure 2 reference message,
        // short enough for the auto-selected M2 symbol.
        Case::new("MicroQR", Symbology::MicroQR, "01234567"),
        // HIBC LIC primary data (labeler code plus product number) from the
        // ANSI/HIBC 2.6 examples; zint prepends the '+' and appends a modulo-43
        // check character.
        Case::new("HIBC128", Symbology::HIBC128, "A123BJC5D6E71"),
        // HIBC LIC secondary data carrying lot and expiry from the ANSI/HIBC 2.6
        // examples; zint prepends the '+' and appends a modulo-43 check character.
        Case::new("HIBC39", Symbology::HIBC39, "$$52001510X3G"),
        // HIBC LIC primary data (labeler H123, product ABC0123456789, unit of
        // measure 0) from ANSI/HIBC LIC Figure C2; zint prepends the + and appends
        // the check character.
        Case::new("HIBCDM", Symbology::HIBCDM, "H123ABC01234567890"),
        // HIBC PAS purchase-order data (unit-of-use plus order number) from
        // ANSI/HIBC PAS Section 2.2; zint prepends the + and appends the check
        // character.
        Case::new("HIBCQR", Symbology::HIBCQR, "/EU720060FF0/O523201"),
        // HIBC LIC primary data (labeler A123, product BJC5D6E7, unit of measure
        // 1); zint prepends the + and appends the check character.
        Case::new("HIBCPDF", Symbology::HIBCPDF, "A123BJC5D6E71"),
        // HIBC PAS secondary data flag from the HIBC Provider Applications
        // Standard; zint prepends the + and appends the check character.
        Case::new("HIBCMicPDF", Symbology::HIBCMicPDF, "/EAH783"),
        // HIBC supplier-labelling string joining primary labeller/product data by
        // "/" to a secondary lot-and-expiry field, with the leading "+" and mod-43
        // check character supplied by Zint.
        Case::new("HIBCCodablockF", Symbology::HIBCCodablockF, "A99912345/$$52001510X3"),
        // HIBC PAS patient identifier with visit timestamp from ANSI/HIBC PAS
        // Section 2.2; zint prepends the + and appends the check character.
        Case::new("HIBCAztec", Symbology::HIBCAztec, "/ACMRN123456/V200912190833"),
        // GS1 element string (GTIN, expiry, batch) from ISS DotCode Rev 4.0 Figure
        // 1, the symbology's main packaging use; GS1 mode needs the [] AI
        // delimiters.
        Case::new("DotCode", Symbology::DotCode, "[01]00012345678905[17]201231[10]ABC123456")
            .input_mode(InputMode::GS1),
        // The ISO 20830 Figure 1 reference message; staying in ASCII avoids the GB
        // 18030 non-compliance warning that Chinese text triggers without an ECI.
        Case::new("HanXin", Symbology::HanXin, "Hanxin Code symbol"),
        // Royal Mail 2D Mailmark: JGB header, supply chain and item IDs,
        // destination AB1 9XY with DPS 1A, service type 0 and return-to-sender
        // postcode AB1 8XY.
        Case::new("Mailmark2D", Symbology::Mailmark2D, "JGB 012100123412345678AB19XY1A 0AB18XY"),
        // UPU S10 item number: EMS service indicator EE, 8-digit serial, mod-11
        // check digit 6 and ISO 3166-1 country code CA.
        Case::new("UPUS10", Symbology::UPUS10, "EE876543216CA"),
        // Royal Mail Mailmark barcode C fields ending in the XY11 international-
        // destination placeholder; Zint appends the trailing spaces.
        Case::new("Mailmark4S", Symbology::Mailmark4S, "1100000000000XY11"),
        // Aztec Runes encode a single whole number 0-255, and 125 is the ISO/IEC
        // 24778 Annex A Figure A.1 example.
        Case::new("AzRune", Symbology::AzRune, "125"),
        // Italian pharmaceutical AIC code; up to 8 digits, check digit added by
        // zint.
        Case::new("Code32", Symbology::Code32, "14352312"),
        // EAN-13 retail article number in the linear part with the item serial
        // number (AI 21) in the CC-A part; zint appends the EAN check digit.
        Case::new("EANXCC", Symbology::EANXCC, "[21]1234-abcd")
            .primary("331234567890"),
        // GS1-128 carrying a GTIN-14 (AI 01) with the unit serial number (AI 21)
        // in the 2D part, per GS1 General Specifications Figure 5.11.8-9.
        Case::new("GS1128CC", Symbology::GS1128CC, "[21]A1B2C3D4E5F6G7H8")
            .primary("[01]03212345678906"),
        // DataBar Omnidirectional GTIN with the production date (AI 11) of 2
        // January 1999 in the 2D part, per GS1 General Specifications Figure
        // 5.11.8-5.
        Case::new("DBarOmnCC", Symbology::DBarOmnCC, "[11]990102")
            .primary("0361234567890"),
        // DataBar Limited GTIN (indicator digit 1, as this symbology only accepts
        // 0 or 1) with expiry date 15 June 2001 and batch A123456 in the 2D part.
        Case::new("DBarLtdCC", Symbology::DBarLtdCC, "[17]010615[10]A123456")
            .primary("1311234567890"),
        // Variable-measure trade item: GTIN plus 1.234 kg net weight (AI 3103) in
        // the linear part, company internal data (AI 91) in the 2D part.
        Case::new("DBarExpCC", Symbology::DBarExpCC, "[91]1A2B3C4D5E")
            .primary("[01]93712345678904[3103]001234"),
        // UPC-A article number from GS1 General Specifications Figure 5.11.8-2
        // with a serial number (AI 21) in the 2D part; the same 2D payload is
        // exercised for UPCA_CC at test_composite.c:3169.
        Case::new("UPCACC", Symbology::UPCACC, "[21]A12345678")
            .primary("61414101234"),
        // Zero-suppressed UPC-E article number with a best-before date (AI 15) of
        // 31 December 2002 in the 2D part.
        Case::new("UPCECC", Symbology::UPCECC, "[15]021231")
            .primary("0121230"),
        // DataBar Stacked GTIN for a small item with an expiry date (AI 17) of end
        // of February 2001 in the 2D part.
        Case::new("DBarStkCC", Symbology::DBarStkCC, "[17]010200")
            .primary("0341234567890"),
        // DataBar Stacked Omnidirectional GTIN with expiry date 1 January 2005 and
        // batch ABC123 in the 2D part.
        Case::new("DBarOmnStkCC", Symbology::DBarOmnStkCC, "[17]050101[10]ABC123")
            .primary("0401234567890"),
        // GTIN plus batch number (AI 10) in the linear part with the unit serial
        // number (AI 21) in the 2D part, per ISO/IEC 24723:2010 Figure 10.
        Case::new("DBarExpStkCC", Symbology::DBarExpStkCC, "[21]12345678")
            .primary("[01]00012345678905[10]ABCDEF"),
        // Six digits select channel 7 by default, and 453678 is under that
        // channel's maximum of 576688.
        Case::new("Channel", Symbology::Channel, "453678"),
        // GS1 element string (GTIN, best-before date, batch) from USS Code One
        // Figure B1, which auto-sizes to Version B; GS1 mode needs the [] AI
        // delimiters.
        Case::new("CodeOne", Symbology::CodeOne, "[01]00312341234014[15]950915[10]ABC123456")
            .input_mode(InputMode::GS1),
        // Charger-IC part number from the AIMD-014 worked example; ASCII only, so
        // no ECI is needed.
        Case::new("GridMatrix", Symbology::GridMatrix, "AAT2556"),
        // Slovenian UPN QR payment order (Example B of the UPN QR technical
        // standard) whose 238 control sum matches these 19 LF-separated fields;
        // Unicode mode maps the Slovenian letters to ISO 8859-2.
        Case::new("UPNQR", Symbology::UPNQR, "UPNQR\nSI56020170014356205\n\n\nSI003528-990\nZdruženje bank Slovenije\nŠubičeva 2\n1000 Ljubljana\n00000128067\n\n\nADVA\nPlačilo avansa-ponudba 2016/12\n\nSI56051008010486080\nSI00123456-67890-12345\nNovo podjetje d.o.o.\nLepa cesta 15\n3698 Loški Potok\n238\n")
            .input_mode(InputMode::UNICODE),
        // Short product URL from the Ultracode specification Figure G.4a, encoded
        // without the optional compression.
        Case::new("Ultra", Symbology::Ultra, "https://aimglobal.org/jcrv3tX"),
        // GS1 element string (GTIN, best-before date, variable count, batch) that
        // rMQR auto-sizes to; GS1 mode needs the [] AI delimiters.
        Case::new("RMQR", Symbology::RMQR, "[01]04912345123459[15]970331[30]128[10]ABC123")
            .input_mode(InputMode::GS1),
        // Semiconductor wafer ID from SEMI T1-95 Figure 2 (7-18 alphanumerics, no
        // letter O); the Rust enum defines this variant as BARCODE_LAST, which
        // happens to equal BARCODE_BC412 (146) today but would drift if upstream
        // adds a symbology.
        Case::new("BC412", Symbology::BC412, "AQ45670"),
    ]
}
