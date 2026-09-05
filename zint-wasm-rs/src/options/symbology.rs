use serde::Deserialize;
use zint_wasm_sys::*;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
#[repr(i32)]
pub enum Symbology {
    Code11 = BARCODE_CODE11 as i32,
    #[serde(alias = "C25Matrix")]
    C25Standard = BARCODE_C25STANDARD as i32,
    C25Inter = BARCODE_C25INTER as i32,
    C25IATA = BARCODE_C25IATA as i32,
    C25Logic = BARCODE_C25LOGIC as i32,
    C25Ind = BARCODE_C25IND as i32,
    Code39 = BARCODE_CODE39 as i32,
    ExCode39 = BARCODE_EXCODE39 as i32,
    EANX = BARCODE_EANX as i32,
    EANXChk = BARCODE_EANX_CHK as i32,
    #[serde(alias = "EAN128")]
    GS1128 = BARCODE_GS1_128 as i32,
    Codabar = BARCODE_CODABAR as i32,
    #[default]
    Code128 = BARCODE_CODE128 as i32,
    DPLEIT = BARCODE_DPLEIT as i32,
    DPIDENT = BARCODE_DPIDENT as i32,
    Code16k = BARCODE_CODE16K as i32,
    Code49 = BARCODE_CODE49 as i32,
    Code93 = BARCODE_CODE93 as i32,
    Flat = BARCODE_FLAT as i32,
    #[serde(alias = "RSS14")]
    DBarOmn = BARCODE_DBAR_OMN as i32,
    #[serde(alias = "RSSLtd")]
    DBarLtd = BARCODE_DBAR_LTD as i32,
    #[serde(alias = "RSSExp")]
    DBarExp = BARCODE_DBAR_EXP as i32,
    Telepen = BARCODE_TELEPEN as i32,
    UPCA = BARCODE_UPCA as i32,
    UPCAChk = BARCODE_UPCA_CHK as i32,
    UPCE = BARCODE_UPCE as i32,
    UPCEChk = BARCODE_UPCE_CHK as i32,
    Postnet = BARCODE_POSTNET as i32,
    MSIPlessey = BARCODE_MSI_PLESSEY as i32,
    FIM = BARCODE_FIM as i32,
    Logmars = BARCODE_LOGMARS as i32,
    Pharma = BARCODE_PHARMA as i32,
    PZN = BARCODE_PZN as i32,
    PharmaTwo = BARCODE_PHARMA_TWO as i32,
    CEPNet = BARCODE_CEPNET as i32,
    PDF417 = BARCODE_PDF417 as i32,
    #[serde(alias = "PDF417Trunc")]
    PDF417Comp = BARCODE_PDF417COMP as i32,
    MaxiCode = BARCODE_MAXICODE as i32,
    QRCode = BARCODE_QRCODE as i32,
    #[serde(alias = "Code128B")]
    Code128AB = BARCODE_CODE128AB as i32,
    AusPost = BARCODE_AUSPOST as i32,
    AusReply = BARCODE_AUSREPLY as i32,
    AusRoute = BARCODE_AUSROUTE as i32,
    AusRedirect = BARCODE_AUSREDIRECT as i32,
    ISBNX = BARCODE_ISBNX as i32,
    RM4SCC = BARCODE_RM4SCC as i32,
    DataMatrix = BARCODE_DATAMATRIX as i32,
    EAN14 = BARCODE_EAN14 as i32,
    VIN = BARCODE_VIN as i32,
    CodablockF = BARCODE_CODABLOCKF as i32,
    NVE18 = BARCODE_NVE18 as i32,
    JapanPost = BARCODE_JAPANPOST as i32,
    KoreaPost = BARCODE_KOREAPOST as i32,
    #[serde(alias = "RSS14Stack")]
    DBarStk = BARCODE_DBAR_STK as i32,
    #[serde(alias = "RSS14StackOmni")]
    DBarOmnStk = BARCODE_DBAR_OMNSTK as i32,
    #[serde(alias = "RSSExpStack")]
    DBarExpStk = BARCODE_DBAR_EXPSTK as i32,
    Planet = BARCODE_PLANET as i32,
    MicroPDF417 = BARCODE_MICROPDF417 as i32,
    #[serde(alias = "OneCode")]
    USPSIMail = BARCODE_USPS_IMAIL as i32,
    Plessey = BARCODE_PLESSEY as i32,
    TelepenNum = BARCODE_TELEPEN_NUM as i32,
    ITF14 = BARCODE_ITF14 as i32,
    KIX = BARCODE_KIX as i32,
    Aztec = BARCODE_AZTEC as i32,
    DAFT = BARCODE_DAFT as i32,
    DPD = BARCODE_DPD as i32,
    MicroQR = BARCODE_MICROQR as i32,
    HIBC128 = BARCODE_HIBC_128 as i32,
    HIBC39 = BARCODE_HIBC_39 as i32,
    HIBCDM = BARCODE_HIBC_DM as i32,
    HIBCQR = BARCODE_HIBC_QR as i32,
    HIBCPDF = BARCODE_HIBC_PDF as i32,
    HIBCMicPDF = BARCODE_HIBC_MICPDF as i32,
    HIBCCodablockF = BARCODE_HIBC_BLOCKF as i32,
    HIBCAztec = BARCODE_HIBC_AZTEC as i32,
    DotCode = BARCODE_DOTCODE as i32,
    HanXin = BARCODE_HANXIN as i32,
    Mailmark2D = BARCODE_MAILMARK_2D as i32,
    UPUS10 = BARCODE_UPU_S10 as i32,
    #[serde(alias = "Mailmark")]
    Mailmark4S = BARCODE_MAILMARK_4S as i32,
    AzRune = BARCODE_AZRUNE as i32,
    Code32 = BARCODE_CODE32 as i32,
    EANXCC = BARCODE_EANX_CC as i32,
    #[serde(alias = "EAN128CC")]
    GS1128CC = BARCODE_GS1_128_CC as i32,
    #[serde(alias = "RSS14CC")]
    DBarOmnCC = BARCODE_DBAR_OMN_CC as i32,
    #[serde(alias = "RSSLtdCC")]
    DBarLtdCC = BARCODE_DBAR_LTD_CC as i32,
    #[serde(alias = "RSSExpCC")]
    DBarExpCC = BARCODE_DBAR_EXP_CC as i32,
    UPCACC = BARCODE_UPCA_CC as i32,
    UPCECC = BARCODE_UPCE_CC as i32,
    #[serde(alias = "RSS14StackCC")]
    DBarStkCC = BARCODE_DBAR_STK_CC as i32,
    #[serde(alias = "RSS14OmniCC")]
    DBarOmnStkCC = BARCODE_DBAR_OMNSTK_CC as i32,
    #[serde(alias = "RSSExpStackCC")]
    DBarExpStkCC = BARCODE_DBAR_EXPSTK_CC as i32,
    Channel = BARCODE_CHANNEL as i32,
    CodeOne = BARCODE_CODEONE as i32,
    GridMatrix = BARCODE_GRIDMATRIX as i32,
    UPNQR = BARCODE_UPNQR as i32,
    Ultra = BARCODE_ULTRA as i32,
    RMQR = BARCODE_RMQR as i32,
    BC412 = BARCODE_BC412 as i32,
}

#[cfg(test)]
mod tests {
    use super::Symbology;
    use crate::{options::Options, test_support::from_cbor};
    use ciborium::cbor;
    use std::{
        collections::{BTreeMap, HashMap, HashSet},
        ffi::CStr,
        os::raw::c_char,
    };
    use zint_wasm_sys::{ZBarcode_BarcodeName, ZBarcode_ValidID};

    /// Declares the symbologies the tests below run over, each with the name
    /// libzint knows it by.
    ///
    /// The generated `match` is what keeps the list honest: adding a variant to
    /// [`Symbology`] without listing it here does not compile.
    macro_rules! symbologies {
        ($($variant:ident => $zint_name:literal),+ $(,)?) => {
            const SYMBOLOGIES: &[(Symbology, &str, &str)] =
                &[$((Symbology::$variant, stringify!($variant), $zint_name)),+];

            #[allow(dead_code)]
            fn every_variant_is_listed(symbology: Symbology) {
                match symbology {
                    $(Symbology::$variant => ()),+
                }
            }
        };
    }

    symbologies![
        Code11 => "BARCODE_CODE11",
        C25Standard => "BARCODE_C25STANDARD",
        C25Inter => "BARCODE_C25INTER",
        C25IATA => "BARCODE_C25IATA",
        C25Logic => "BARCODE_C25LOGIC",
        C25Ind => "BARCODE_C25IND",
        Code39 => "BARCODE_CODE39",
        ExCode39 => "BARCODE_EXCODE39",
        EANX => "BARCODE_EANX",
        EANXChk => "BARCODE_EANX_CHK",
        GS1128 => "BARCODE_GS1_128",
        Codabar => "BARCODE_CODABAR",
        Code128 => "BARCODE_CODE128",
        DPLEIT => "BARCODE_DPLEIT",
        DPIDENT => "BARCODE_DPIDENT",
        Code16k => "BARCODE_CODE16K",
        Code49 => "BARCODE_CODE49",
        Code93 => "BARCODE_CODE93",
        Flat => "BARCODE_FLAT",
        DBarOmn => "BARCODE_DBAR_OMN",
        DBarLtd => "BARCODE_DBAR_LTD",
        DBarExp => "BARCODE_DBAR_EXP",
        Telepen => "BARCODE_TELEPEN",
        UPCA => "BARCODE_UPCA",
        UPCAChk => "BARCODE_UPCA_CHK",
        UPCE => "BARCODE_UPCE",
        UPCEChk => "BARCODE_UPCE_CHK",
        Postnet => "BARCODE_POSTNET",
        MSIPlessey => "BARCODE_MSI_PLESSEY",
        FIM => "BARCODE_FIM",
        Logmars => "BARCODE_LOGMARS",
        Pharma => "BARCODE_PHARMA",
        PZN => "BARCODE_PZN",
        PharmaTwo => "BARCODE_PHARMA_TWO",
        CEPNet => "BARCODE_CEPNET",
        PDF417 => "BARCODE_PDF417",
        PDF417Comp => "BARCODE_PDF417COMP",
        MaxiCode => "BARCODE_MAXICODE",
        QRCode => "BARCODE_QRCODE",
        Code128AB => "BARCODE_CODE128AB",
        AusPost => "BARCODE_AUSPOST",
        AusReply => "BARCODE_AUSREPLY",
        AusRoute => "BARCODE_AUSROUTE",
        AusRedirect => "BARCODE_AUSREDIRECT",
        ISBNX => "BARCODE_ISBNX",
        RM4SCC => "BARCODE_RM4SCC",
        DataMatrix => "BARCODE_DATAMATRIX",
        EAN14 => "BARCODE_EAN14",
        VIN => "BARCODE_VIN",
        CodablockF => "BARCODE_CODABLOCKF",
        NVE18 => "BARCODE_NVE18",
        JapanPost => "BARCODE_JAPANPOST",
        KoreaPost => "BARCODE_KOREAPOST",
        DBarStk => "BARCODE_DBAR_STK",
        DBarOmnStk => "BARCODE_DBAR_OMNSTK",
        DBarExpStk => "BARCODE_DBAR_EXPSTK",
        Planet => "BARCODE_PLANET",
        MicroPDF417 => "BARCODE_MICROPDF417",
        USPSIMail => "BARCODE_USPS_IMAIL",
        Plessey => "BARCODE_PLESSEY",
        TelepenNum => "BARCODE_TELEPEN_NUM",
        ITF14 => "BARCODE_ITF14",
        KIX => "BARCODE_KIX",
        Aztec => "BARCODE_AZTEC",
        DAFT => "BARCODE_DAFT",
        DPD => "BARCODE_DPD",
        MicroQR => "BARCODE_MICROQR",
        HIBC128 => "BARCODE_HIBC_128",
        HIBC39 => "BARCODE_HIBC_39",
        HIBCDM => "BARCODE_HIBC_DM",
        HIBCQR => "BARCODE_HIBC_QR",
        HIBCPDF => "BARCODE_HIBC_PDF",
        HIBCMicPDF => "BARCODE_HIBC_MICPDF",
        HIBCCodablockF => "BARCODE_HIBC_BLOCKF",
        HIBCAztec => "BARCODE_HIBC_AZTEC",
        DotCode => "BARCODE_DOTCODE",
        HanXin => "BARCODE_HANXIN",
        Mailmark2D => "BARCODE_MAILMARK_2D",
        UPUS10 => "BARCODE_UPU_S10",
        Mailmark4S => "BARCODE_MAILMARK_4S",
        AzRune => "BARCODE_AZRUNE",
        Code32 => "BARCODE_CODE32",
        EANXCC => "BARCODE_EANX_CC",
        GS1128CC => "BARCODE_GS1_128_CC",
        DBarOmnCC => "BARCODE_DBAR_OMN_CC",
        DBarLtdCC => "BARCODE_DBAR_LTD_CC",
        DBarExpCC => "BARCODE_DBAR_EXP_CC",
        UPCACC => "BARCODE_UPCA_CC",
        UPCECC => "BARCODE_UPCE_CC",
        DBarStkCC => "BARCODE_DBAR_STK_CC",
        DBarOmnStkCC => "BARCODE_DBAR_OMNSTK_CC",
        DBarExpStkCC => "BARCODE_DBAR_EXPSTK_CC",
        Channel => "BARCODE_CHANNEL",
        CodeOne => "BARCODE_CODEONE",
        GridMatrix => "BARCODE_GRIDMATRIX",
        UPNQR => "BARCODE_UPNQR",
        Ultra => "BARCODE_ULTRA",
        RMQR => "BARCODE_RMQR",
        BC412 => "BARCODE_BC412",
    ];

    /// Zint's public header, so the tests below can ask what it defines rather
    /// than only what this crate claims about it.
    const ZINT_H: &str = include_str!("../../../zint-wasm-sys/zint/backend/zint.h");

    /// The former name of a renamed symbology, the alias that still accepts it,
    /// and the variant it names now. Zint keeps its own former names, marked
    /// `Legacy` in the header, and the test below holds the two lists together.
    const LEGACY_NAMES: &[(&str, &str, Symbology)] = &[
        ("BARCODE_C25MATRIX", "C25Matrix", Symbology::C25Standard),
        ("BARCODE_EAN128", "EAN128", Symbology::GS1128),
        ("BARCODE_RSS14", "RSS14", Symbology::DBarOmn),
        ("BARCODE_RSS_LTD", "RSSLtd", Symbology::DBarLtd),
        ("BARCODE_RSS_EXP", "RSSExp", Symbology::DBarExp),
        ("BARCODE_PDF417TRUNC", "PDF417Trunc", Symbology::PDF417Comp),
        ("BARCODE_CODE128B", "Code128B", Symbology::Code128AB),
        ("BARCODE_RSS14STACK", "RSS14Stack", Symbology::DBarStk),
        (
            "BARCODE_RSS14STACK_OMNI",
            "RSS14StackOmni",
            Symbology::DBarOmnStk,
        ),
        ("BARCODE_RSS_EXPSTACK", "RSSExpStack", Symbology::DBarExpStk),
        ("BARCODE_ONECODE", "OneCode", Symbology::USPSIMail),
        ("BARCODE_MAILMARK", "Mailmark", Symbology::Mailmark4S),
        ("BARCODE_EAN128_CC", "EAN128CC", Symbology::GS1128CC),
        ("BARCODE_RSS14_CC", "RSS14CC", Symbology::DBarOmnCC),
        ("BARCODE_RSS_LTD_CC", "RSSLtdCC", Symbology::DBarLtdCC),
        ("BARCODE_RSS_EXP_CC", "RSSExpCC", Symbology::DBarExpCC),
        (
            "BARCODE_RSS14STACK_CC",
            "RSS14StackCC",
            Symbology::DBarStkCC,
        ),
        (
            "BARCODE_RSS14_OMNI_CC",
            "RSS14OmniCC",
            Symbology::DBarOmnStkCC,
        ),
        (
            "BARCODE_RSS_EXPSTACK_CC",
            "RSSExpStackCC",
            Symbology::DBarExpStkCC,
        ),
    ];

    /// One `#define BARCODE_...` line in zint's header.
    #[derive(Debug, PartialEq, Eq)]
    struct ZintSymbology<'a> {
        constant: &'a str,
        value: i32,
        /// Zint marks the former name of a renamed symbology `Legacy` and keeps
        /// it defined alongside the new one, at the same value.
        legacy: bool,
    }

    /// The symbologies zint's header defines, in the order it defines them.
    ///
    /// Input:  the lines
    ///         `#define BARCODE_C25STANDARD     2   /* 2 of 5 Standard (Matrix) */`
    ///         `#define BARCODE_C25MATRIX       2   /* Legacy */`
    /// Output: `BARCODE_C25STANDARD` at 2, and `BARCODE_C25MATRIX` at 2 as a
    ///         legacy name
    fn zint_symbologies(header: &str) -> Vec<ZintSymbology<'_>> {
        let mut defines = Vec::new();
        let mut reached_marker = false;

        for line in header.lines() {
            // `trim_end` because the checkout's line endings are not ours to
            // choose, and a carriage return would land inside the comment.
            let Some(rest) = line.trim_end().strip_prefix("#define ") else {
                continue;
            };
            let mut fields = rest.split_whitespace();
            let (Some(constant), Some(value)) = (fields.next(), fields.next()) else {
                continue;
            };
            if !constant.starts_with("BARCODE_") {
                continue;
            }
            // Everything past the marker is an output option sharing the
            // prefix, not a symbology.
            if constant == "BARCODE_LAST" {
                reached_marker = true;
                break;
            }
            let Ok(value) = value.parse::<i32>() else {
                continue;
            };

            defines.push(ZintSymbology {
                constant,
                value,
                // The comment alone, so that a marker zint spells differently
                // reads as a live symbology and is reported as one missing a
                // variant, rather than being quietly dropped from the check.
                legacy: comment(rest) == Some("Legacy"),
            });
        }

        // The two below are what stop this from reporting success because it
        // understood nothing. A header that declares its symbologies some other
        // way, or that no longer marks where they end, is the change these
        // tests exist to notice.
        assert!(
            reached_marker,
            "zint.h no longer marks the end of its symbologies with BARCODE_LAST"
        );
        assert!(
            !defines.is_empty(),
            "no `#define BARCODE_...` lines were found in zint.h"
        );
        defines
    }

    /// The body of a define's trailing block comment, if it has one.
    ///
    /// Input:  `BARCODE_C25MATRIX       2   /* Legacy */`
    /// Output: `Some("Legacy")`
    fn comment(define: &str) -> Option<&str> {
        let (_, rest) = define.split_once("/*")?;
        let (body, _) = rest.split_once("*/")?;
        Some(body.trim())
    }

    /// Reads back the name libzint knows a symbology by, which is how these
    /// tests check that a variant still points at the encoder it is named
    /// after.
    ///
    /// Input:  `Symbology::Code128` (id 20)
    /// Output: `Some("BARCODE_CODE128")`
    fn zint_name(symbology: Symbology) -> Option<String> {
        let mut name = [0 as c_char; 32];
        let unknown = unsafe {
            // Safety: the C signature asks for a 32 byte buffer, which is what
            // `name` is.
            ZBarcode_BarcodeName(symbology as i32, name.as_mut_ptr())
        };
        if unknown != 0 {
            return None;
        }
        let name = unsafe {
            // Safety: zint terminates the name it wrote and never fills the
            // buffer completely.
            CStr::from_ptr(name.as_ptr())
        };
        Some(name.to_string_lossy().into_owned())
    }

    #[test]
    fn every_symbology_is_one_zint_knows() {
        for (symbology, variant, _) in SYMBOLOGIES {
            let valid = unsafe {
                // Safety: any `int` is a valid argument; zint range checks it.
                ZBarcode_ValidID(*symbology as i32)
            };
            assert_ne!(
                valid, 0,
                "{variant} ({}) is not a symbology zint can encode",
                *symbology as i32
            );
        }
    }

    /// The identity check that makes a libzint upgrade reviewable: every
    /// variant has to still resolve to the encoder it is named after, not just
    /// to some valid number.
    #[test]
    fn every_symbology_points_at_the_encoder_it_is_named_after() {
        for (symbology, variant, expected) in SYMBOLOGIES {
            assert_eq!(
                zint_name(*symbology).as_deref(),
                Some(*expected),
                "{variant} ({}) resolves to the wrong zint symbology",
                *symbology as i32
            );
        }
    }

    /// Zint to Rust, which is the direction nothing else checks. Every other
    /// test here starts from a [`Symbology`] and asks zint about it, so a
    /// symbology zint gains is invisible to all of them: it simply never comes
    /// up. Reading the header is what makes the submodule moving under the
    /// crate say so, rather than the new symbology being quietly unavailable.
    #[test]
    fn every_symbology_zint_defines_has_a_variant() {
        let bound: HashSet<&str> = SYMBOLOGIES.iter().map(|(_, _, zint)| *zint).collect();

        let missing: Vec<&str> = zint_symbologies(ZINT_H)
            .iter()
            .filter(|it| !it.legacy)
            .map(|it| it.constant)
            .filter(|constant| !bound.contains(constant))
            .collect();

        assert!(
            missing.is_empty(),
            "zint defines these symbologies and `Symbology` has no variant for them: {}",
            missing.join(", ")
        );
    }

    /// The other half of the same agreement. Zint keeps the former name of a
    /// renamed symbology so that older callers keep working, and this crate
    /// keeps the same names as serde aliases so that older documents do. Both
    /// lists have to move together, and the value ties each pair to the
    /// symbology it actually names rather than to a plausible-looking one.
    #[test]
    fn every_legacy_name_zint_keeps_is_still_accepted() {
        let defined = zint_symbologies(ZINT_H);
        // Ordered, so a failure lists the constants the same way twice running.
        let legacy: BTreeMap<&str, i32> = defined
            .iter()
            .filter(|it| it.legacy)
            .map(|it| (it.constant, it.value))
            .collect();
        let listed: HashSet<&str> = LEGACY_NAMES
            .iter()
            .map(|(constant, _, _)| *constant)
            .collect();
        assert_eq!(
            listed.len(),
            LEGACY_NAMES.len(),
            "a constant is listed twice, which hides one that is not listed at all"
        );

        let unhandled: Vec<&str> = legacy
            .keys()
            .copied()
            .filter(|constant| !listed.contains(constant))
            .collect();
        assert!(
            unhandled.is_empty(),
            "zint still answers to these former names and nothing here accepts them: {}",
            unhandled.join(", ")
        );

        for (constant, alias, expected) in LEGACY_NAMES {
            let value = legacy
                .get(constant)
                .unwrap_or_else(|| panic!("zint no longer keeps {constant}"));
            assert_eq!(
                *value, *expected as i32,
                "{constant} is {value} in zint, but {alias} is accepted as {expected:?}"
            );

            let parsed: Options = from_cbor(cbor!({ "symbology" => *alias }).unwrap())
                .unwrap_or_else(|error| panic!("{alias} is not accepted: {error}"));
            assert_eq!(parsed.symbology as i32, *expected as i32, "{alias}");
        }
    }

    /// The shapes the parser has to tell apart, so that the two tests above
    /// cannot report success by having understood nothing.
    const SAMPLE_HEADER: &str = concat!(
        "/* Symbologies (`symbol->symbology`) */\n",
        "    /* Tbarcode 7 codes */\n",
        "#define BARCODE_CODE11          1   /* Code 11 */\n",
        "#define BARCODE_C25STANDARD     2   /* 2 of 5 Standard (Matrix) */\n",
        "#define BARCODE_C25MATRIX       2   /* Legacy */\n",
        "#define ZINT_MAX_DATA_LEN       17400\n",
        "#define BARCODE_LAST            146 /* Max barcode number marker */\n",
        "\n/* Output options (`symbol->output_options`) */\n",
        "#define BARCODE_BIND            0x00002 /* Boundary bars */\n",
    );

    #[test]
    fn the_header_parse_reads_what_it_claims_to() {
        let parsed = zint_symbologies(SAMPLE_HEADER);

        assert_eq!(
            parsed,
            vec![
                ZintSymbology {
                    constant: "BARCODE_CODE11",
                    value: 1,
                    legacy: false,
                },
                ZintSymbology {
                    constant: "BARCODE_C25STANDARD",
                    value: 2,
                    legacy: false,
                },
                ZintSymbology {
                    constant: "BARCODE_C25MATRIX",
                    value: 2,
                    legacy: true,
                },
            ],
            "the output option after the marker, and the unrelated define, are not symbologies"
        );
    }

    /// Without the marker the parser cannot tell a symbology from an output
    /// option, and quietly returning the ones it did find would let the
    /// coverage tests pass on a header they no longer understand.
    #[test]
    #[should_panic(expected = "no longer marks the end of its symbologies")]
    fn a_header_without_the_marker_is_refused() {
        zint_symbologies("#define BARCODE_CODE11          1   /* Code 11 */\n");
    }

    #[test]
    #[should_panic(expected = "no `#define BARCODE_...` lines were found")]
    fn a_header_that_declares_symbologies_some_other_way_is_refused() {
        zint_symbologies("enum { BARCODE_CODE11 = 1 };\n#define BARCODE_LAST 146\n");
    }

    /// Git hands a Windows checkout CRLF unless something pins the file, and
    /// nothing pins the vendored header, so it has to parse either way.
    #[test]
    fn the_header_parses_whatever_line_endings_the_checkout_used() {
        let unix = ZINT_H.replace("\r\n", "\n");
        let windows = unix.replace('\n', "\r\n");

        assert_eq!(zint_symbologies(&unix), zint_symbologies(&windows));
    }

    #[test]
    fn no_two_symbologies_share_an_id() {
        let mut seen: HashMap<i32, &str> = HashMap::new();
        for (symbology, variant, _) in SYMBOLOGIES {
            if let Some(other) = seen.insert(*symbology as i32, variant) {
                panic!("{variant} and {other} are both {}", *symbology as i32);
            }
        }
    }

    /// Typst documents the symbology by its Rust name, so the name a document
    /// carries and the variant name have to stay the same string.
    #[test]
    fn every_symbology_deserializes_from_its_own_name() {
        for (symbology, variant, _) in SYMBOLOGIES {
            let parsed: Options = from_cbor(cbor!({ "symbology" => *variant }).unwrap())
                .unwrap_or_else(|error| panic!("{variant} is not accepted: {error}"));

            assert_eq!(parsed.symbology as i32, *symbology as i32, "{variant}");
        }
    }

    #[test]
    fn an_unknown_symbology_is_rejected() {
        let error = from_cbor::<Options>(cbor!({ "symbology" => "Code129" }).unwrap())
            .expect_err("Code129 does not exist");
        assert!(error.contains("Code129"), "unexpected error: {error}");
    }

    #[test]
    fn the_default_symbology_is_code128() {
        assert_eq!(
            Symbology::default() as i32,
            Symbology::Code128 as i32,
            "the default has to stay the one the Typst package documents"
        );
    }
}
