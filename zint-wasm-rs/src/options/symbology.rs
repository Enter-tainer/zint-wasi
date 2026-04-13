use serde::Deserialize;
use std::{ffi::CString, fmt::Display};
use zint_rs_macros::symbol_data;
use zint_sys::*;
use crate::options::{capability::CapabilityFlags, GenericOptions};
use crate::options::values;
use crate::segment::ECI;

// `symbol_data!` macro is responsible for generating `Symbology` enum as well
// as each symbol options or a type alias to another symbol options that are
// identical.
//
// Each symbol is defined as: `SymbolName = { ...options... }`
//
// Currently supported options are:
// - raw
// - alias
// - options
// - apply_options
//
// # `raw` option
// Stores a zint constant associated with the symbol, this value will be used
// for both `Symbology` and `SymbolOptions` enum discriminants. This provides a
// 1:1 mapping to c implementation and between these enums to ensure
// consistency.
//
// # `alias` option
// Can be either a string or an array of strings which are alternative names for
// the symbol. The bindings will often use a more generic name for symbology is
// possible in which case these aliases provide alternative deserialization
// names to avoid confusion when users are attempting to use symbols based on
// original zint names.
//
// These aliases are also recognized by Symbology::from_str.
//
// Aliases can also contain obsolete names for symbologies, or alternative names
// when a symbology is used under different names in different sectors/regions.
// Canonical name in this case is often chosen arbitrarily, and doesn't indicate
// any endorsement of use-cases or governments.
//
// For instance `EAN5` symbology is defined as `BARCODE_EAN_5ADDON` in zint.
// Name "EAN5" is chosen because "Addon" part is not necessary to identify what
// symbology the enum variant refers to. Therefore "EAN5" is deemed a better
// choice as canonical name of the symbology in question because "Addon" part
// isn't necessary to differentiate it from other variants (i.e. there's no
// non-addon EAN5 variant).
//
// # `options` option
// This option is used to specify options that are used when encoding a barcode.
// It can reference an existing struct with format:
//
// - `options: StructName`, where the macro will generate `<SymbolName>Options`
//   type alias to `StructName`,
// - `options: StructName as CustomName`, where the macro will generate `type
//   CustomName = StructName` alias and use `CustomName` in `SymbolOptions`
//   enum,
// - or `options: StructName as _`, where the macro will simply use `StructName`
//   in generated code instead of generating a type alias.
//
// `StructName` can also specify a single lifetime which must be then be used in
// `CustomName` if provided.
//
// The `options` can alternatively be an inline struct declaration with format:
//
// ```
// options: [NameOverride] [<'any_options_lifetime_name>] {
//   ... option fields ...
// }
// ```
//
// This option accepts rustdoc and other attributes, which will be forwarded to
// generated options struct or type alias.
//
// # `apply_options` option
// Provides a "closure" that will be used to convert the type specified by
// `options` into `GenericOptions`. This isn't a real closure, it only allows
// specifying 2 argument names `result: &mut GenericOptions` and `options:
// &TypeSpecifiedWithOptions`, followed by an expression.
//
// The expression code will be turned into `ConfigureSymbolOptions::configure`
// function.
//
// When `options` option isn't a struct declaration, `apply_options` option is
// not needed because the specified type is expected to implement
// `ConfigureSymbolOptions`.
//
// ---
//
// The symbol_data! macro is not perfect and doesn't cover a lot of edge-cases,
// it's meant for internal use only and functionality covers only the needs of
// zint-rs. Features are added as-needed.

symbol_data! {
    #[allow(clippy::upper_case_acronyms)]
    #[derive(Debug, Clone, Copy, Default, Deserialize)]
    #[non_exhaustive]
    #[repr(i32)]
    pub enum Symbology {
        /// Code 11
        Code11 = {
            raw: BARCODE_CODE11,
            kebab_case: "code11",
            options: {
                /// Number of modulo-11 check digits to add to the symbol.
                check_digits: usize = 2,
            },
            apply_options: |result, options| {
                result.option_2 = Some(match options.check_digits {
                    0 => 2,
                    1 => 1,
                    2 => 0,
                    other => {
                        return Err(SymbolOptionError {
                            option_name: "check_digits",
                            value: other.to_string(),
                            reason: "more than 2 check digits not supported by Code 11".to_string()
                        });
                    }
                });
                Ok(())
            },
        },
        /// 2 of 5 Standard (Matrix)
        C25Standard = {
            raw: BARCODE_C25STANDARD,
            kebab_case: "c25-standard",
            options: C25Options {
                /// Whether to add a check digit to 2 of 5 code.
                check_digit: bool,
                /// Whether to hide the check digit from Human Readable Text (HRT).
                hide_check_digit: bool,
            },
            apply_options: |result, options| {
                result.option_2 = Some(match (options.check_digit, options.hide_check_digit) {
                    (true, false) => 1,
                    (true, true) => 2,
                    _ => 0,
                });
                Ok(())
            }
        },
        /// 2 of 5 Interleaved
        C25Inter = {
            raw: BARCODE_C25INTER,
            kebab_case: "c25-inter",
            options: C25Options,
        },
        /// 2 of 5 IATA
        C25IATA = {
            raw: BARCODE_C25IATA,
            kebab_case: "c25-iata",
            options: C25Options,
        },
        /// 2 of 5 Data Logic
        C25Logic = {
            raw: BARCODE_C25LOGIC,
            kebab_case: "c25-logic",
            options: C25Options,
        },
        /// 2 of 5 Industrial
        C25Ind = {
            raw: BARCODE_C25IND,
            kebab_case: "c25-ind",
            options: C25Options,
        },
        /// Code 39
        Code39 = {
            raw: BARCODE_CODE39,
            kebab_case: "code39",
        },
        /// Extended Code 39
        ExCode39 = {
            raw: BARCODE_EXCODE39,
            kebab_case: "ex-code39",
        },
        /// Variable length EAN variant
        #[deprecated(note = "will be removed; use specialized EAN code variants")]
        EANX = {
            raw: BARCODE_EANX,
            category: "retail",
            options: UPCEOptions,
        },
        /// Variable length EAN variant with digit check
        #[deprecated(note = "will be removed; use specialized EAN CHK code variants")]
        EANXChk = {
            raw: BARCODE_EANX_CHK,
            category: "retail",
            options: UPCEOptions,
        },
        /// Variable length composite EAN variant
        #[deprecated(note = "will be removed; use specialized EAN CC code variants")]
        EANXCC = {
            raw: BARCODE_EANX_CC,
            kebab_case: "eanx-cc",
            category: "retail",
            options: UPCEOptions,
        },
        /// EAN/UPC 2-digit
        /// 
        /// This symbology is almost never used as standalone and is generally
        /// appended to other one-dimensional barcodes.
        EAN2 = {
            raw: BARCODE_EAN_2ADDON,
            category: "retail",
            alias: "EAN2Addon",
            options: UPCEOptions,
        },
        /// EAN/UPC 5-digit
        /// 
        /// This symbology is almost never used as standalone and is generally
        /// appended to other one-dimensional barcodes.
        EAN5 = {
            raw: BARCODE_EAN_5ADDON,
            category: "retail",
            alias: "EAN5Addon",
            options: UPCEOptions,
        },
        /// EAN-8 (European Article Number) GTIN-8
        /// 
        /// In addition EAN-2 and EAN-5 add-on symbols can be added by using the
        /// '+' character as a separator after EAN-8 data.
        EAN8 = {
            raw: BARCODE_EAN8,
            category: "retail",
            alias: "GTIN8",
            options: UPCEOptions,
        },
        /// EAN-13 (European Article Number) is a standard 13-digit barcode used
        /// in retail across Europe.
        /// 
        /// EAN-13 requires 12-digit numerical input. The check digit is
        /// calculated by Zint.
        /// 
        /// In addition EAN-2 and EAN-5 add-on symbols can be added by using the
        /// '+' character as a separator after EAN-13 data.
        EAN13 = {
            raw: BARCODE_EAN13,
            category: "retail",
            alias: "GTIN13",
            options: UPCEOptions,
        },
        /// EAN-14
        /// 
        /// EAN-14 requires 13-digit numerical input. The check digit is
        /// calculated by Zint.
        /// 
        /// In addition EAN-2 and EAN-5 add-on symbols can be added by using the
        /// '+' character as a separator after EAN-14 data.
        EAN14 = {
            raw: BARCODE_EAN14,
            category: "retail",
            alias: "GTIN14",
            options: UPCEOptions,
        },
        /// EAN-8 Composite
        EAN8CC = {
            raw: BARCODE_EAN8_CC,
            category: "retail",
            options: UPCEOptions,
        },
        /// EAN-13 Composite
        EAN13CC = {
            raw: BARCODE_EAN13_CC,
            category: "retail",
            options: UPCEOptions,
        },
        /// GS1-128
        GS1128 = {
            raw: BARCODE_GS1_128,
            kebab_case: "gs1-128",
            category: "gs1",
            alias: "EAN128",
        },
        /// Codabar
        Codabar = {
            raw: BARCODE_CODABAR,
        },
        /// Code 128
        #[default]
        Code128 = {
            raw: BARCODE_CODE128,
            kebab_case: "code128",
        },
        /// Deutsche Post Leitcode, used by Deutsche Post for mail routing and
        /// sorting. Based on Interleaved Code 2 of 5.
        ///
        /// Input must be exactly 13 decimal digits (0-9). Zint automatically
        /// computes and appends the modulo-10 check digit, producing a
        /// 14-digit encoded value.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        DPLEIT = {
            raw: BARCODE_DPLEIT,
            category: "postal",
        },
        /// Deutsche Post Identcode, used by Deutsche Post for mail item
        /// identification. Based on Interleaved Code 2 of 5.
        ///
        /// Input must be exactly 11 decimal digits (0-9). Zint automatically
        /// computes and appends the modulo-10 check digit, producing a
        /// 12-digit encoded value.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        DPIDENT = {
            raw: BARCODE_DPIDENT,
            category: "postal",
        },
        /// Code 16k
        Code16k = {
            raw: BARCODE_CODE16K,
            kebab_case: "code16k",
        },
        /// Code 49
        Code49 = {
            raw: BARCODE_CODE49,
            kebab_case: "code49",
        },
        /// Code 93
        Code93 = {
            raw: BARCODE_CODE93,
            kebab_case: "code93",
        },
        /// Flattermarken
        Flat = {
            raw: BARCODE_FLAT,
        },
        /// GS1 DataBar Omnidirectional
        DBarOmn = {
            raw: BARCODE_DBAR_OMN,
            kebab_case: "dbar-omn",
            category: "gs1",
            alias: "RSS14"
        },
        /// GS1 DataBar Limited
        DBarLtd = {
            raw: BARCODE_DBAR_LTD,
            kebab_case: "dbar-ltd",
            category: "gs1",
        },
        /// GS1 DataBar Expanded
        DBarExp = {
            raw: BARCODE_DBAR_EXP,
            kebab_case: "dbar-exp",
            category: "gs1",
        },
        /// Telepen Alpha
        Telepen = {
            raw: BARCODE_TELEPEN,
        },
        /// UPC-A is used in the United States for retail applications. The
        /// symbol requires an 11-digit article number. The check digit is
        /// calculated by Zint.
        ///
        /// In addition EAN-2 and EAN-5 add-on symbols can be added by using the
        /// '+' character as a separator before EAN-2/EAN-5 data.
        ///
        /// If input data already includes the check digit,
        /// [`UPCAChk`][Symbology::UPCAChk] can be used.
        UPCA = {
            raw: BARCODE_UPCA,
            category: "retail",
            options: {
                /// Gap between the main symbol and an add-on in multiples of the X-dimension.
                addon_gap: usize = 9,
                /// Height in X-dimensions that the guard bars descend below the main bars.
                guard_descent: f32 = 5.0,
            },
            apply_options: |result, options| {
                result.option_2 = Some(require_range_inclusive("addon_gap", options.addon_gap as std::ffi::c_int, 9, 12)?);
                result.guard_descent =
                    require_range_inclusive("guard_descent", options.guard_descent, 0.0, 20.0)?;
                Ok(())
            }
        },
        /// UPC-A variant that expects check digit to be included in input data.
        ///
        /// See [`UPCA`][Symbology::UPCA] for details.
        UPCAChk = {
            raw: BARCODE_UPCA_CHK,
            category: "retail",
            options: UPCAOptions,
        },
        /// UPC-E
        UPCE = {
            raw: BARCODE_UPCE,
            category: "retail",
            options: {
                /// Gap between the main symbol and an add-on in multiples of the X-dimension.
                addon_gap: usize = 7,
                /// Height in X-dimensions that the guard bars descend below the main bars.
                guard_descent: f32 = 5.0,
            },
            apply_options: |result, options| {
                result.option_2 = Some(require_range_inclusive("addon_gap", options.addon_gap as std::ffi::c_int, 7, 12)?);
                result.guard_descent =
                    require_range_inclusive("guard_descent", options.guard_descent, 0.0, 20.0)?;
                Ok(())
            }
        },
        /// UPC-E including check digit
        UPCEChk = {
            raw: BARCODE_UPCE_CHK,
            category: "retail",
            options: UPCEOptions,
        },
        /// USPS POSTNET (Postal Numeric Encoding Technique), used by the
        /// United States Postal Service until 2009 to encode ZIP codes on
        /// mail items.
        ///
        /// Accepts numerical input (digits 0-9) up to 38 digits. Zint
        /// automatically adds the modulo-10 check digit. Standard lengths
        /// used by USPS were:
        ///
        /// - 5 digits (ZIP code only, `PostNet6`)
        /// - 9 digits (ZIP+4, `PostNet10`)
        /// - 11 digits (ZIP+4+delivery point, `PostNet12`)
        ///
        /// Zint will issue a warning if the input length is not one of these
        /// standard lengths.
        ///
        /// Superseded by [`USPSIMail`][Symbology::USPSIMail] (Intelligent
        /// Mail) in 2009.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        Postnet = {
            raw: BARCODE_POSTNET,
            category: "postal",
        },
        /// MSI Plessey is based on [`Plessey`][Symbology::Plessey] and
        /// developed by MSE Data Corporation.
        /// 
        /// MSI Plessey has a range of check digit options that are selectable
        /// by setting [`check_digits`][MSIPlesseyOptions::check_digits].
        /// 
        /// Numeric (digits 0-9) input can be encoded, up to a maximum of 65
        /// digits.
        MSIPlessey = {
            raw: BARCODE_MSI_PLESSEY,
            options: {
                /// Specifies mode for generated check digits.
                check_digits: values::MSIPlesseyCheckDigits,
                /// Whether to hide the check digit from Human Readable Text (HRT).
                hide_check_digit: bool,
            },
            apply_options: |result, options| {
                result.option_2 = Some({
                    let mut value: std::ffi::c_int = options.check_digits.into();
                    if options.hide_check_digit {
                        value += 10;
                    }
                    value
                });
                Ok(())
            },
        },
        /// Facing Identification Mark
        FIM = {
            raw: BARCODE_FIM,
        },
        /// LOGMARS
        LOGMARS = {
            raw: BARCODE_LOGMARS,
            alias: "Logmars"
        },
        /// Pharmacode One-Track, developed by Laetus for pharmaceutical
        /// product identification.
        ///
        /// Encodes whole numbers between 3 and 131070 inclusive. Input is
        /// the numeric value as a decimal string.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        ///
        Pharma = {
            raw: BARCODE_PHARMA,
            category: "healthcare",
        },
        /// Pharmazentralnummer (PZN) — a Code 39 based symbology used by
        /// the pharmaceutical industry in Germany for product identification.
        ///
        /// By default encodes in PZN8 format (current standard since 2013):
        /// a 7-digit number to which a modulo-11 check digit is appended.
        /// Inputs shorter than 7 digits are zero-padded. An 8-digit input
        /// is accepted in which case Zint validates the check digit.
        ///
        /// PZN7 format (obsolete since 2013) can be selected via
        /// [`version`][PZNOptions::version].
        ///
        PZN = {
            raw: BARCODE_PZN,
            category: "healthcare",
            options: {
                /// Selects PZN format version.
                ///
                /// Defaults to [`PZN8`][values::PZNVersion::PZN8] (current
                /// standard). Set to [`PZN7`][values::PZNVersion::PZN7] for
                /// the obsolete 7-digit format.
                version: values::PZNVersion,
            },
            apply_options: |result, options| {
                if options.version == values::PZNVersion::PZN7 {
                    result.option_2 = Some(1);
                }
                Ok(())
            },
        },
        /// Pharmacode Two-Track, developed by Laetus as an alternative to
        /// Pharmacode One-Track for pharmaceutical product identification.
        ///
        /// Encodes whole numbers between 4 and 64570080 inclusive. Input is
        /// the numeric value as a decimal string.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        ///
        PharmaTwo = {
            raw: BARCODE_PHARMA_TWO,
            category: "healthcare",
        },
        /// Brazilian CEPNet Postal Code, used by Correios (the Brazilian
        /// postal service) to encode CEP (Código de Endereçamento Postal)
        /// numbers on mail items.
        ///
        /// Based on POSTNET. Input must be exactly 8 decimal digits; Zint
        /// automatically adds the modulo-10 check digit.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        CEPNet = {
            raw: BARCODE_CEPNET,
            kebab_case: "cepnet",
            category: "postal",
        },
        /// PDF417
        PDF417 = {
            raw: BARCODE_PDF417,
            kebab_case: "pdf417",
            category: "2d",
        },
        /// Compact PDF417 (Truncated PDF417)
        PDF417Comp = {
            raw: BARCODE_PDF417COMP,
            kebab_case: "pdf417-comp",
            category: "2d",
            alias: "PDF417Trunc",
        },
        /// MaxiCode symbology is designed for the identification of parcels.
        /// 
        /// MaxiCode symbols can be encoded in one of five modes which are
        /// specified using the [`mode`][MaxiCodeOptions::mode] option.
        /// 
        /// This symbology uses Latin-1 character encoding by default but also
        /// supports the ECI encoding mechanism. The maximum length of text
        /// which can be placed in a MaxiCode symbol depends on the type of
        /// characters used in the text and `mode`, see
        /// [`MaxiCodeMode`][values::MaxiCodeMode] variants for details.
        MaxiCode = {
            raw: BARCODE_MAXICODE,
            kebab_case: "maxicode",
            category: "2d",
            options: <'o> {
                /// MaxiCode mode and corresponding options to use for encoding.
                /// 
                /// See [`MaxiCodeMode`][values::MaxiCodeMode] for details.
                mode: values::MaxiCodeMode<'o>,
                /// 2-digit version used in **ISO/IEC 15434 Format 01
                /// (transportation)** prefix `[)>\R01\Gvv` which will be
                /// prepended to ASCII-compatible secondary message data.
                /// 
                /// This prefix will only be used with
                /// [`StructuredCarrierMessage`][values::MaxiCodeMode::StructuredCarrierMessage]
                /// and will be ignored otherwise.
                /// 
                /// Only values in range [0, 99] are valid, and zint will return
                /// an error if provided data ECI isn't ASCII-compatible.
                /// 
                /// This is only a utility option and prefix can be included in
                /// secondary message data manually.
                scm_prefix: Option<u8>,
            },
            apply_options: |result, options| {
                if let Some(primary) = options.mode.primary_message() {
                    if primary.eci() != ECI::ISO_8859_1 {
                        return Err(SymbolOptionError {
                            option_name: "mode",
                            value: format!("{primary:?}"),
                            reason: "primary message must use ISO 8859-1 (Latin-1) ECI".to_string(),
                        });
                    }
                    result.primary_message = Some(primary);
                }
                if let Some(mode) = options.mode.mode() {
                    result.option_1 = Some(mode as std::ffi::c_int);
                } else if let Some(scm_prefix) = options.scm_prefix {
                    let scm_prefix = require_range_inclusive("scm_prefix", scm_prefix, 0, 99)?;
                    // Will still return invalid option error if data isn't
                    // using an ASCII-compatible ECI. Can't check that as it
                    // isn't known yet.
                    result.option_2 = Some((scm_prefix + 1) as std::ffi::c_int);
                }
                Ok(())
            }
        },
        /// QR Code
        QRCode = {
            raw: BARCODE_QRCODE,
            kebab_case: "qrcode",
            category: "2d",
            options: {
                error_correction: Option<values::QRErrorCorrection>,
                size: Option<values::QRSize>,
                full_multibyte: bool,
                mask: Option<values::QRMask>,
            },
            apply_options: |result, options| {
                if let Some(error_correction) = options.error_correction {
                    result.option_1 = Some(error_correction as std::ffi::c_int);
                }
                if let Some(size) = options.size {
                    result.option_2 = Some(Into::<u8>::into(size) as std::ffi::c_int);
                }

                let mut option_3_value = 0;
                if options.full_multibyte {
                    option_3_value |= ZINT_FULL_MULTIBYTE;
                }
                if let Some(mask) = options.mask {
                    option_3_value |= ((mask as u32) + 1) << 8;
                }
                if option_3_value != 0 {
                    result.option_3 = Some(option_3_value as std::ffi::c_int);
                }

                Ok(())
            }
        },
        /// Code 128 (Suppress Code Set C)
        Code128AB = {
            raw: BARCODE_CODE128AB,
            kebab_case: "code128ab",
            alias: "CODE128B"
        },
        /// Australia Post Standard Customer Barcode (4-State), used to print
        /// Delivery Point ID (DPID) and optional customer information on mail
        /// items. Format Control Code (FCC) is added by Zint automatically.
        ///
        /// Valid input characters are `0-9`, `A-Z`, `a-z`, space, and `#`.
        /// Input length determines the symbol length and encoding table used:
        ///
        /// - 8 digits → 37-bar symbol (FCC 11, numeric DPID only)
        /// - 13 chars (`NNNNNNNNAAAAAA`) → 52-bar symbol
        /// - 16 digits → 67-bar symbol (FCC 59)
        /// - 18 chars (`NNNNNNNNAAAAAAAAA`) → 52-bar symbol
        /// - 23 digits → 67-bar symbol (FCC 62)
        ///
        /// Reed-Solomon error correction data is generated automatically.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        AusPost = {
            raw: BARCODE_AUSPOST,
            category: "postal",
        },
        /// Australia Post Reply Paid Barcode (4-State, FCC 45).
        ///
        /// A specialised Australia Post 4-state barcode for reply-paid mail.
        /// Requires exactly 8 decimal digits representing the DPID.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        AusReply = {
            raw: BARCODE_AUSREPLY,
            category: "postal",
        },
        /// Australia Post Routing Barcode (4-State, FCC 87).
        ///
        /// A specialised Australia Post 4-state barcode for mail routing.
        /// Requires exactly 8 decimal digits representing the DPID.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        AusRoute = {
            raw: BARCODE_AUSROUTE,
            category: "postal",
        },
        /// Australia Post Redirection Barcode (4-State, FCC 92).
        ///
        /// A specialised Australia Post 4-state barcode for mail redirection.
        /// Requires exactly 8 decimal digits representing the DPID.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        AusRedirect = {
            raw: BARCODE_AUSREDIRECT,
            category: "postal",
        },
        /// ISBN
        /// 
        /// Relevant check digit needs to be present in the input data and will
        /// be verified before the symbol is generated.
        /// 
        /// In addition EAN-2 and EAN-5 add-on symbols can be added using the +
        /// character as with UPC symbols.
        ISBNX = {
            raw: BARCODE_ISBNX,
            category: "retail",
            options: UPCEOptions,
        },
        /// Royal Mail 4-State Customer Code (RM4SCC), used by Royal Mail in
        /// the UK to encode postcode and customer data on mail items.
        ///
        /// Input may consist of digits `0-9` and uppercase letters `A-Z`,
        /// typically formatted as the delivery postcode followed by the house
        /// number. Zint automatically computes and appends the check digit.
        ///
        /// Superseded by [`Mailmark4S`][Symbology::Mailmark4S] (Royal Mail
        /// 4-State Mailmark) in 2014, which adds Reed-Solomon error
        /// correction.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        RM4SCC = {
            raw: BARCODE_RM4SCC,
            kebab_case: "rm4scc",
            category: "postal",
        },
        /// Data Matrix (ECC200)
        DataMatrix = {
            raw: BARCODE_DATAMATRIX,
            category: "2d",
        },
        /// Vehicle Identification Number
        VIN = {
            raw: BARCODE_VIN,
        },
        /// Codablock-F
        CodablockF = {
            raw: BARCODE_CODABLOCKF,
            category: "2d",
        },
        /// NVE-18 (SSCC-18)
        NVE18 = {
            raw: BARCODE_NVE18,
            kebab_case: "nve18",
            category: "retail",
        },
        /// Japanese Postal Code (4-State), used for address data on mail
        /// items by Japan Post.
        ///
        /// Accepted input characters are `0-9`, `A-Z`, and dash (`-`). A
        /// modulo-19 check digit is computed and appended automatically by
        /// Zint.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        JapanPost = {
            raw: BARCODE_JAPANPOST,
            category: "postal",
        },
        /// Korea Post Barcode, used by the Korean postal service.
        ///
        /// Encodes a 6-digit numeric code. Zint automatically computes and
        /// appends one check digit, producing a 7-digit encoded value.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        KoreaPost = {
            raw: BARCODE_KOREAPOST,
            category: "postal",
        },
        /// GS1 DataBar Stacked
        DBarStk = {
            raw: BARCODE_DBAR_STK,
            kebab_case: "dbar-stk",
            category: "gs1",
            alias: "RSS14Stk"
        },
        /// GS1 DataBar Stacked Omnidirectional
        DBarOmnStk = {
            raw: BARCODE_DBAR_OMNSTK,
            kebab_case: "dbar-omn-stk",
            category: "gs1",
            alias: "RSS14StackOmni"
        },
        /// GS1 DataBar Expanded Stacked
        #[serde(alias = "RSSExpStack")]
        DBarExpStk = {
            raw: BARCODE_DBAR_EXPSTK,
            kebab_case: "dbar-exp-stk",
            category: "gs1",
        },
        /// USPS PLANET (Postal Alpha Numeric Encoding Technique), used by the
        /// United States Postal Service until 2009 to encode routing data on
        /// mail items.
        ///
        /// Accepts numerical input (digits 0-9) up to 38 digits. Zint
        /// automatically adds the modulo-10 check digit. Standard lengths
        /// used by USPS were:
        ///
        /// - 11 digits (`Planet12`)
        /// - 13 digits (`Planet14`)
        ///
        /// Zint will issue a warning if the input length is not one of these
        /// standard lengths.
        ///
        /// Superseded by [`USPSIMail`][Symbology::USPSIMail] (Intelligent
        /// Mail) in 2009.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        Planet = {
            raw: BARCODE_PLANET,
            category: "postal",
        },
        /// MicroPDF417
        MicroPDF417 = {
            raw: BARCODE_MICROPDF417,
            kebab_case: "micro-pdf417",
            category: "2d",
        },
        /// USPS Intelligent Mail (also known as OneCode), used by the United
        /// States Postal Service since 2009 as a replacement for both
        /// [`Postnet`][Symbology::Postnet] and [`Planet`][Symbology::Planet].
        ///
        /// A fixed-length 65-bar symbol combining routing and customer
        /// information. Input data consists of a 20-digit tracking code
        /// followed by a dash (`-`) and a delivery point ZIP code of 0, 5,
        /// 9, or 11 digits. The following input lengths are all valid:
        ///
        /// - `"01234567094987654321"` (20 digits, no ZIP)
        /// - `"01234567094987654321-01234"` (20 + 5 digits)
        /// - `"01234567094987654321-012345678"` (20 + 9 digits)
        /// - `"01234567094987654321-01234567891"` (20 + 11 digits)
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        USPSIMail = {
            raw: BARCODE_USPS_IMAIL,
            kebab_case: "usps-imail",
            category: "postal",
            alias: "OneCode"
        },
        /// Plessey (Code) symbology was developed by the Plessey Company Ltd.
        /// in the UK.
        /// 
        /// The symbol can encode data consisting of digits (0-9) or letters A-F
        /// up to a maximum of 65 characters and includes a CRC check digit.
        Plessey = {
            raw: BARCODE_PLESSEY,
        },

        // Tbarcode 8 codes
        /// Telepen Numeric
        TelepenNum = {
            raw: BARCODE_TELEPEN_NUM,
        },
        /// ITF-14, also known as UPC Shipping Container Symbol or Case Code, is
        /// based on Interleaved Code 2 of 5 and requires a 13-digit numeric
        /// input (digits 0-9).
        ITF14 = {
            raw: BARCODE_ITF14,
            kebab_case: "itf14",
            category: "retail",
        },
        /// Dutch Post KIX Code, used by Royal Dutch TPG Post (PostNL,
        /// Netherlands) for postal code and automatic mail sorting.
        ///
        /// Input must be exactly 11 characters consisting of digits `0-9`
        /// and uppercase letters `A-Z`. No check digit is included.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        KIX = {
            raw: BARCODE_KIX,
            category: "postal",
        },
        /// Aztec Code
        Aztec = {
            raw: BARCODE_AZTEC,
            category: "2d",
        },
        /// DAFT Code — a generic 4-state barcode format where the data
        /// encoding is supplied by an external program.
        ///
        /// Input must consist only of the characters `D`, `A`, `F`, and `T`,
        /// which represent the four bar states:
        ///
        /// - `D` — Descender (bar extends below the baseline only)
        /// - `A` — Ascender (bar extends above the baseline only)
        /// - `F` — Full bar (ascender and descender)
        /// - `T` — Tracker (neither ascender nor descender; baseline only)
        ///
        /// The ratio of the tracker height to the full bar height can be
        /// configured via [`tracker_ratio`][DAFTOptions::tracker_ratio].
        DAFT = {
            raw: BARCODE_DAFT,
            category: "postal",
            options: {
                /// Ratio of the tracker bar height to the full bar height,
                /// specified in permille (thousandths). Valid range is 1 to
                /// 999. Defaults to 250 (25%).
                ///
                /// For example, a value of 256 makes tracker bars 25.6% of
                /// the full bar height.
                tracker_ratio: Option<u16>,
            },
            apply_options: |result, options| {
                if let Some(ratio) = options.tracker_ratio {
                    result.option_2 = Some(
                        require_range_inclusive("tracker_ratio", ratio, 1u16, 999)? as std::ffi::c_int
                    );
                }
                Ok(())
            },
        },
        /// DPD Code, a variant of Code 128 used by DPD (Deutscher
        /// Paketdienst) for parcel identification.
        ///
        /// Requires a 27 or 28 character input. For 28-character input, the
        /// first character is an identification tag (Barcode ID), which should
        /// usually be `%` (ASCII 37). If 27 characters are supplied, `%` is
        /// prepended automatically (unless relabel mode is active). The
        /// remaining 27-character body must be alphanumeric and structured as:
        ///
        /// - 7 alphanumeric characters: destination post code
        /// - 14 alphanumeric characters: tracking number
        /// - 3 digits: service code
        /// - 3 characters (ISO 3166-1 numeric): destination country code
        ///
        /// Zint formats the Human Readable Text per the DPD specification,
        /// omitting the identification tag and appending a modulo-36 check
        /// character. A top boundary bar is added by default.
        ///
        /// The [`relabel`][DPDOptions::relabel] option omits the identification
        /// tag and prints the barcode at half height. In this case exactly 27
        /// alphanumeric input characters are required.
        DPD = {
            raw: BARCODE_DPD,
            category: "postal",
            options: {
                /// Marks the symbol as a "relabel" barcode.
                ///
                /// When `true`, the identification tag (`%`) is omitted and
                /// the barcode is printed at half height. In relabel mode
                /// exactly 27 alphanumeric input characters are required
                /// (no 28-character form with explicit Barcode ID).
                ///
                /// Defaults to `false`.
                relabel: bool,
            },
            apply_options: |result, options| {
                if options.relabel {
                    result.option_2 = Some(1);
                }
                Ok(())
            },
        },
        /// Micro QR Code
        MicroQR = {
            raw: BARCODE_MICROQR,
            kebab_case: "micro-qr",
            category: "2d",
        },

        // Tbarcode 9 codes
        /// HIBC Code 128 — Health Industry Barcode (HIBC) variant of Code 128.
        ///
        /// Automatically prepends a `'+'` character and appends a modulo-49
        /// check digit to a standard Code 128 symbol, as required by the
        /// Health Industry Barcode Council (HIBCC) standard.
        ///
        /// Supports full ASCII input (same character set as Code 128). This
        /// is a pass-through encoding wrapper: no additional options are
        /// available beyond those on the input data itself.
        HIBC128 = {
            raw: BARCODE_HIBC_128,
            category: "healthcare",
        },
        /// HIBC Code 39 — Health Industry Barcode (HIBC) variant of Code 39.
        ///
        /// Automatically prepends a `'+'` character and appends a modulo-49
        /// check digit to a standard Code 39 symbol, as required by the
        /// Health Industry Barcode Council (HIBCC) standard.
        ///
        /// Supports the standard Code 39 character set (A-Z, 0-9, and
        /// `-`, `.`, ` `, `$`, `/`, `+`, `%`). This is a pass-through
        /// encoding wrapper: no additional options are available.
        HIBC39 = {
            raw: BARCODE_HIBC_39,
            category: "healthcare",
        },
        /// HIBC Data Matrix — Health Industry Barcode (HIBC) variant of
        /// Data Matrix (ECC200).
        ///
        /// Automatically prepends a `'+'` character and appends a modulo-49
        /// check digit as required by the HIBCC standard. Only ECC 200
        /// symbols are supported (the older ECC 000-140 formats have been
        /// removed from zint).
        ///
        /// The symbol size can be set using [`size`][HIBCDMOptions::size]
        /// (1-30 for standard, 31-48 for DMRE rectangular). The automatic
        /// size selection shape can be controlled with
        /// [`shape`][HIBCDMOptions::shape].
        HIBCDM = {
            raw: BARCODE_HIBC_DM,
            kebab_case: "hibc-dm",
            category: "healthcare",
            options: {
                /// Symbol size (1-30 for standard, 31-48 for DMRE rectangular
                /// extension). Set to `None` for automatic size selection.
                ///
                /// See the zint manual Table 25 and Table 26 for a full list
                /// of symbol sizes.
                size: Option<u8>,
                /// Shape preference for automatic symbol size selection.
                ///
                /// Ignored when `size` is explicitly set.
                shape: values::DataMatrixShape,
            },
            apply_options: |result, options| {
                if let Some(size) = options.size {
                    let size = require_range_inclusive("size", size, 1u8, 48)?;
                    result.option_2 = Some(size as std::ffi::c_int);
                }
                match options.shape {
                    values::DataMatrixShape::Any => {}
                    values::DataMatrixShape::Square => {
                        result.option_3 = Some(DM_SQUARE as std::ffi::c_int);
                    }
                    values::DataMatrixShape::AllowDMRE => {
                        result.option_3 = Some(DM_DMRE as std::ffi::c_int);
                    }
                }
                Ok(())
            },
        },
        /// HIBC QR Code — Health Industry Barcode (HIBC) variant of QR Code.
        ///
        /// Automatically prepends a `'+'` character and appends a modulo-49
        /// check digit as required by the HIBCC standard.
        ///
        /// Supports the same encoding options as standard QR Code: error
        /// correction level, symbol version (size), full-multibyte mode, and
        HIBCQR = {
            raw: BARCODE_HIBC_QR,
            kebab_case: "hibc-qr",
            category: "healthcare",
            options: QRCodeOptions,
        },
        /// HIBC PDF417 — Health Industry Barcode (HIBC) variant of PDF417.
        ///
        /// Automatically prepends a `'+'` character and appends a modulo-49
        /// check digit as required by the HIBCC standard.
        ///
        /// Supports the same layout options as standard PDF417: number of
        /// columns (1-30), rows (3-90), and error correction level (0-8).
        HIBCPDF = {
            raw: BARCODE_HIBC_PDF,
            kebab_case: "hibc-pdf",
            category: "healthcare",
            options: {
                /// Error correction level.
                ///
                /// Number of error correction codewords equals `2^(level + 1)`.
                /// Set to `None` to let zint choose automatically based on
                /// data length.
                error_correction: Option<values::PDF417ErrorCorrection>,
                /// Number of data columns (1-30). Set to `None` for automatic.
                columns: Option<u8>,
                /// Number of rows (3-90). Set to `None` for automatic.
                rows: Option<u8>,
            },
            apply_options: |result, options| {
                if let Some(ecc) = options.error_correction {
                    result.option_1 = Some(ecc as std::ffi::c_int);
                }
                if let Some(cols) = options.columns {
                    result.option_2 = Some(
                        require_range_inclusive("columns", cols, 1u8, 30)? as std::ffi::c_int
                    );
                }
                if let Some(rows) = options.rows {
                    result.option_3 = Some(
                        require_range_inclusive("rows", rows, 3u8, 90)? as std::ffi::c_int
                    );
                }
                Ok(())
            },
        },
        /// HIBC MicroPDF417 — Health Industry Barcode (HIBC) variant of
        /// MicroPDF417.
        ///
        /// Automatically prepends a `'+'` character and appends a modulo-49
        /// check digit as required by the HIBCC standard.
        ///
        /// MicroPDF417 is a compact variant of PDF417 with fixed error
        /// correction (determined by symbol size). The number of data columns
        /// (1-4) can be set; the number of rows is determined automatically
        /// by the amount of data.
        ///
        /// Maximum capacity is 250 alphanumeric characters or 366 digits.
        HIBCMicroPDF = {
            raw: BARCODE_HIBC_MICPDF,
            kebab_case: "hibc-micro-pdf",
            category: "healthcare",
            alias: "HIBCMicPDF",
            options: {
                /// Number of data columns (1-4). Set to `None` for automatic
                /// selection based on data length.
                columns: Option<values::MicroPDF417Columns>,
            },
            apply_options: |result, options| {
                if let Some(cols) = options.columns {
                    result.option_2 = Some(cols as std::ffi::c_int);
                }
                Ok(())
            },
        },
        /// HIBC Codablock-F — Health Industry Barcode (HIBC) variant of
        /// Codablock-F.
        ///
        /// Automatically prepends a `'+'` character and appends a modulo-49
        /// check digit to the encoded data as required by the HIBCC standard.
        ///
        /// Codablock-F is a stacked Code 128 symbology. The symbol width
        /// (number of columns, 9-67) and height (number of rows, 1-44) can
        /// be configured.
        HIBCCodablockF = {
            raw: BARCODE_HIBC_BLOCKF,
            category: "healthcare",
            alias: "HIBCBlockF",
            options: {
                /// Number of rows (1-44). Set to `None` for automatic
                /// selection based on data length.
                rows: Option<u8>,
                /// Number of data columns (9-67). Set to `None` for automatic
                /// selection.
                columns: Option<u8>,
            },
            apply_options: |result, options| {
                if let Some(rows) = options.rows {
                    result.option_1 = Some(
                        require_range_inclusive("rows", rows, 1u8, 44)? as std::ffi::c_int
                    );
                }
                if let Some(cols) = options.columns {
                    result.option_2 = Some(
                        require_range_inclusive("columns", cols, 9u8, 67)? as std::ffi::c_int
                    );
                }
                Ok(())
            },
        },
        /// HIBC Aztec Code — Health Industry Barcode (HIBC) variant of
        /// Aztec Code (ISO 24778).
        ///
        /// Automatically prepends a `+` character and appends a modulo-49
        /// check digit as required by the HIBCC standard.
        ///
        /// Two mutually exclusive options control symbol sizing:
        /// - [`error_correction`][HIBCAztecOptions::error_correction] — set
        ///   the minimum error correction level (1-4); zint selects the
        ///   smallest symbol that meets it.
        /// - [`size`][HIBCAztecOptions::size] — pin the exact symbol version
        ///   (1-36, where 1-4 are compact symbols); `error_correction` is
        ///   ignored when `size` is set.
        ///
        /// By default zint targets ≥23% error correction and chooses symbol
        /// type and size automatically.
        HIBCAztec = {
            raw: BARCODE_HIBC_AZTEC,
            category: "healthcare",
            options: {
                /// Minimum error correction level.
                ///
                /// Ignored when [`size`][HIBCAztecOptions::size] is set.
                /// Set to `None` for the default (~23% + 3 codewords).
                error_correction: Option<values::AztecErrorCorrection>,
                /// Explicit symbol size version (1-36).
                ///
                /// Versions 1-4 are compact symbols; 5-36 are full-range.
                /// When set, [`error_correction`][HIBCAztecOptions::error_correction]
                /// is ignored.
                size: Option<values::AztecSize>,
            },
            apply_options: |result, options| {
                if let Some(size) = options.size {
                    result.option_2 = Some(size.version() as std::ffi::c_int);
                } else if let Some(ecc) = options.error_correction {
                    result.option_1 = Some(ecc as std::ffi::c_int);
                }
                Ok(())
            },
        },

        // Tbarcode 10 codes
        /// DotCode
        DotCode = {
            raw: BARCODE_DOTCODE,
            kebab_case: "dotcode",
            category: "2d",
        },
        /// Han Xin (Chinese Sensible) Code
        HanXin = {
            raw: BARCODE_HANXIN,
            kebab_case: "hanxin",
            category: "2d",
        },

        // Tbarcode 11 codes
        /// Royal Mail 2D Mailmark (CMDM — Complex Mail Data Mark), a Data
        /// Matrix-based barcode introduced by Royal Mail alongside the
        /// 4-State Mailmark.
        ///
        /// Input is a pre-formatted string with an initial 45-character
        /// section followed by optional customer data. Zint will prepend
        /// `"JGB "` if absent and space-pad the customer data as needed.
        ///
        /// The mandatory 45-character section encodes:
        /// - UPU Country ID (4 chars, e.g. `"JGB "`)
        /// - Information Type (1 alphanumeric)
        /// - Version ID (1 char, `"1"`)
        /// - Class (1 alphanumeric)
        /// - Supply Chain ID (7 digits)
        /// - Item ID (8 digits)
        /// - Destination+DPS (9 alphanumeric, one of 13 patterns)
        /// - Service Type (1 digit)
        /// - RTS Post Code (7 alphanumeric, one of 7 patterns)
        /// - Reserved (6 spaces)
        ///
        /// Three symbol sizes are defined, differing in customer data
        /// capacity, selectable via [`size`][Mailmark2DOptions::size]:
        ///
        /// | Type | Size    | Customer Data | Zint version |
        /// |------|---------|---------------|--------------|
        /// | 7    | 24×24   | 6 characters  | 8            |
        /// | 9    | 32×32   | 45 characters | 10           |
        /// | 29   | 16×48   | 29 characters | 30           |
        ///
        /// Zint selects the smallest size that fits the customer data
        /// automatically. GS1 data, ECI, and Structured Append are not
        /// supported.
        Mailmark2D = {
            raw: BARCODE_MAILMARK_2D,
            kebab_case: "mailmark-2d",
            category: "postal",
            options: {
                /// Symbol size to use.
                ///
                /// Corresponds to the Zint version number (one more than
                /// the Royal Mail Type number):
                /// - `Some(8)` → Type 7 (24×24, 6 chars customer data)
                /// - `Some(10)` → Type 9 (32×32, 45 chars customer data)
                /// - `Some(30)` → Type 29 (16×48, 29 chars customer data)
                /// - `None` → automatic selection based on customer data
                ///   length (rectangular Type 29 may be excluded via
                ///   [`shape`][Mailmark2DOptions::shape])
                size: Option<values::Mailmark2DSize>,
                /// Shape preference for automatic symbol size selection.
                ///
                /// When set to [`Square`][values::DataMatrixShape::Square],
                /// the rectangular Type 29 (16×48) symbol is excluded from
                /// automatic selection. Has no effect when
                /// [`size`][Mailmark2DOptions::size] is explicitly set.
                shape: values::DataMatrixShape,
            },
            apply_options: |result, options| {
                if let Some(size) = options.size {
                    result.option_2 = Some(size as std::ffi::c_int);
                }
                match options.shape {
                    values::DataMatrixShape::Any => {}
                    values::DataMatrixShape::Square => {
                        result.option_3 = Some(DM_SQUARE as std::ffi::c_int);
                    }
                    values::DataMatrixShape::AllowDMRE => {
                        result.option_3 = Some(DM_DMRE as std::ffi::c_int);
                    }
                }
                Ok(())
            },
        },
        /// Universal Postal Union S10, a Code 128-based format for
        /// international mail item identification standardised by the UPU.
        ///
        /// Input must be 13 characters in the format `SSNNNNNNNNXCC`, where:
        /// - `SS` — two uppercase alphabetic characters (service indicator)
        /// - `NNNNNNNN` — eight-digit serial number
        /// - `X` — one modulo-11 check digit (may be omitted; Zint adds it)
        /// - `CC` — two uppercase alphabetic characters (ISO 3166-1 country code)
        ///
        /// If the check digit (`X`) is omitted (12-character input), Zint
        /// computes and inserts it. Warnings are generated if the service
        /// indicator is non-standard or the country code is not a recognised
        /// ISO 3166-1 code.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        UPUS10 = {
            raw: BARCODE_UPU_S10,
            kebab_case: "upus10",
            category: "postal",
        },
        /// Royal Mail 4-State Mailmark, introduced in 2014 as a replacement
        /// for [`RM4SCC`][Symbology::RM4SCC] with added Reed-Solomon error
        /// correction.
        ///
        /// Input is a pre-formatted alphanumeric string of either 22 or 26
        /// characters:
        /// - **22 characters** → Barcode C (66-bar symbol)
        /// - **26 characters** → Barcode L (78-bar symbol)
        ///
        /// The input fields are:
        /// - Format (1 digit, 0-4)
        /// - Version ID (1 digit, 0-3)
        /// - Class (1 alphanumeric, `0-9A-E`)
        /// - Supply Chain ID (2 digits for C, 6 digits for L)
        /// - Item ID (8 digits)
        /// - Destination+DPS (9 alphanumeric, one of 6 defined patterns)
        ///
        /// Trailing space characters in the Destination+DPS field are
        /// appended automatically if not included. The fixed string `"XY11 "`
        /// designates an international destination.
        ///
        /// This symbology has no configurable options; all encoding is
        /// determined by the input data.
        Mailmark4S = {
            raw: BARCODE_MAILMARK_4S,
            kebab_case: "mailmark-4s",
            category: "postal",
            alias: "Mailmark"
        },

        // Zint specific codes
        /// Aztec Runes
        AzRune = {
            raw: BARCODE_AZRUNE,
            kebab_case: "azrune",
            category: "2d",
        },
        /// Code 32
        Code32 = {
            raw: BARCODE_CODE32,
            kebab_case: "code32",
        },
        /// GS1-128 Composite
        GS1128CC = {
            raw: BARCODE_GS1_128_CC,
            kebab_case: "gs1-128-cc",
            category: "gs1",
            alias: "EAN128CC",
        },
        /// GS1 DataBar Omnidirectional Composite
        DBarOmnCC = {
            raw: BARCODE_DBAR_OMN_CC,
            kebab_case: "dbar-omn-cc",
            category: "gs1",
            alias: "RSS14CC",
        },
        /// GS1 DataBar Limited Composite
        DBarLtdCC = {
            raw: BARCODE_DBAR_LTD_CC,
            kebab_case: "dbar-ltd-cc",
            category: "gs1",
            alias: "RSSLtdCC",
        },
        /// GS1 DataBar Expanded Composite
        DBarExpCC = {
            raw: BARCODE_DBAR_EXP_CC,
            kebab_case: "dbar-exp-cc",
            category: "gs1",
            alias: "RSSExpCC",
        },
        /// UPC-A Composite
        UPCACC = {
            raw: BARCODE_UPCA_CC,
            kebab_case: "upca-cc",
            category: "retail",
        },
        /// UPC-E Composite
        UPCECC = {
            raw: BARCODE_UPCE_CC,
            kebab_case: "upce-cc",
            category: "retail",
        },
        /// GS1 DataBar Stacked Composite
        DBarStkCC = {
            raw: BARCODE_DBAR_STK_CC,
            kebab_case: "dbar-stk-cc",
            category: "gs1",
            alias: "RSS14StackCC",
        },
        /// GS1 DataBar Stacked Omnidirectional Composite
        DBarOmnStkCC = {
            raw: BARCODE_DBAR_OMNSTK_CC,
            kebab_case: "dbar-omn-stk-cc",
            category: "gs1",
            alias: "RSS14OmniCC",
        },
        /// GS1 DataBar Expanded Stacked Composite
        DBarExpStkCC = {
            raw: BARCODE_DBAR_EXPSTK_CC,
            kebab_case: "dbar-exp-stk-cc",
            category: "gs1",
            alias: "RSSExpStackCC",
        },
        /// Channel Code
        Channel = {
            raw: BARCODE_CHANNEL,
        },
        /// Code One
        CodeOne = {
            raw: BARCODE_CODEONE,
            kebab_case: "code-one",
            category: "2d",
        },
        /// Grid Matrix
        GridMatrix = {
            raw: BARCODE_GRIDMATRIX,
            category: "2d",
        },
        /// UPNQR (Univerzalnega Plačilnega Naloga QR — Universal Payment
        /// Order QR), a QR Code variant used by Združenje Bank Slovenije
        /// (Bank Association of Slovenia) for payment orders.
        ///
        /// A fixed-size QR Code (version 15, 77×77 modules) with a fixed
        /// error correction level (M) and ECI 4 (ISO 8859-2). These
        /// parameters are set internally by Zint and cannot be overridden.
        ///
        /// Input data is Latin-2 (ISO/IEC 8859-2 plus ASCII) encoded. Zint
        /// accepts UTF-8 and converts it to Latin-2 automatically. If the
        /// data is already Latin-2 encoded, use binary input mode
        /// (`input_mode = DATA_MODE`).
        UPNQR = {
            raw: BARCODE_UPNQR,
            category: "postal",
            options: {
                /// Manually specify the QR mask pattern. When not set, zint
                /// selects the optimal mask automatically.
                mask: Option<values::QRMask>,
            },
            apply_options: |result, options| {
                if let Some(mask) = options.mask {
                    result.option_3 = Some(((mask as u32 + 1) << 8) as std::ffi::c_int);
                }
                Ok(())
            }
        },
        /// Ultracode symbology uses a grid of coloured elements to encode data.
        /// 
        /// ECI and GS1 modes are supported.
        Ultra = {
            raw: BARCODE_ULTRA,
            category: "2d",
            options: {
                /// Specifies amount of symbol holding error correction data.
                error_correction: values::UltracodeErrorCorrection,

                /// Enables support for data compression.
                /// 
                /// ## Experimental
                /// 
                /// Ultracode data compression is experimental and should not be
                /// used in a production environment. 
                compression: bool,

                /// Specifies which Ultracode revision to use for symbol
                /// encoding.
                revision: values::UltracodeRevision,
            },
            apply_options: |result, options| {
                result.option_1 = Some(options.error_correction as std::ffi::c_int);
                if options.revision.0 > 0 {
                    result.option_2 = Some((options.revision.0 + 1) as std::ffi::c_int);
                }
                if options.compression {
                    result.option_3 = Some(ULTRA_COMPRESSION as std::ffi::c_int);
                }
                Ok(())
            }
        },
        /// Rectangular Micro QR Code (rMQR)
        RMQR = {
            raw: BARCODE_RMQR,
            kebab_case: "rmqr",
            category: "2d",
        },
        /// IBM BC412 (SEMI T1-95)
        BC412 = {
            raw: BARCODE_BC412,
            kebab_case: "bc412",
        },
        /// DX Film Edge Barcode on 35mm and APS films
        DXFilmEdge = {
            raw: BARCODE_DXFILMEDGE,
        },
    }
}

impl Symbology {
    /// Returns source name of the symbol.
    ///
    /// This name is derived from `BARCODE_*` constants, and excludes `BARCODE_`
    /// prefix.
    ///
    /// Use [`Display`] trait to get human readable names instead.
    ///
    /// Example: `Symbology::HIBCMicroPDF` which is internally
    /// `BARCODE_HIBC_MICPDF` returns a "HIBC_MICPDF" string.
    pub fn zint_name(self) -> String {
        let mut read_buffer = vec![0; 32];
        let result =
            unsafe { ZBarcode_BarcodeName(self as std::ffi::c_int, read_buffer.as_mut_ptr() as *mut i8) };
        if result == 1 {
            panic!("Symbology value is invalid");
        }
        // SAFETY: zint always insterts a nul byte at the end
        let read_buffer = unsafe { CString::from_vec_with_nul_unchecked(read_buffer) };
        let result = unsafe { read_buffer.to_str().unwrap_unchecked() };
        result.to_string()
    }

    /// Returns default width in mm for this symbology.
    pub fn default_width(self) -> f32 {
        unsafe { ZBarcode_Default_Xdim(self as std::ffi::c_int) }
    }

    /// Returns the scale needed for this symbol to be `xdim` X-dimension
    /// (in milimeters) at `dots_per_mm`.
    ///
    /// If `xdim` or `dots_per_mm` are zero, negative or too large, a
    /// [`UnitConversionError`] is returned.
    ///
    /// Target `filetype` affects scaling due to inherent precision limitations
    /// of different formats.
    ///
    /// If any of the arguments are invalid, 0 is returned.
    pub fn scale_from_xdim_and_dpmm(
        self,
        xdim: f32,
        dots_per_mm: f32,
        filetype: TargetFiletype,
    ) -> Result<f32, UnitConversionError> {
        if xdim <= 0.0 || xdim > 10.0 {
            return Err(UnitConversionError::InvalidXDimension(xdim));
        }
        if dots_per_mm <= 0.0 || dots_per_mm > 1000.0 {
            return Err(UnitConversionError::InvalidDpmm(dots_per_mm));
        }
        Ok(unsafe {
            ZBarcode_Scale_From_XdimDp(self as std::ffi::c_int, xdim, dots_per_mm, filetype.as_c_str().as_ptr())
        })
    }

    /// Returns estimate X-dimension (in milimeters) for given `scale` and
    /// `dots_per_mm`.
    ///
    /// If `scale` or `dots_per_mm` are zero, negative or too large, a
    /// [`UnitConversionError`] is returned.
    ///
    /// If any of the arguments are invalid, 0 is returned.
    pub fn xdim_from_scale_and_dpmm(
        self,
        scale: f32,
        dots_per_mm: f32,
        filetype: TargetFiletype,
    ) -> Result<f32, UnitConversionError> {
        // These ranges return 0 values from the zint library. They are wrapped
        // in UnitConversionError to aid in better error handling:
        if scale <= 0.0 || scale > 200.0 {
            return Err(UnitConversionError::InvalidScale(scale));
        }
        if dots_per_mm <= 0.0 || dots_per_mm > 1000.0 {
            return Err(UnitConversionError::InvalidDpmm(dots_per_mm));
        }
        Ok(unsafe {
            ZBarcode_XdimDp_From_Scale(
                self as std::ffi::c_int,
                scale,
                dots_per_mm,
                filetype.as_c_str().as_ptr(),
            )
        })
    }

    /// Returns required dots per milimeter to draw a barcode of specified
    /// `xdim` X-dimension (in milimeters), at a given `scale`.
    ///
    /// If `scale` or `xdim` are zero, negative or too large, a
    /// [`UnitConversionError`] is returned.
    ///
    /// If any of the arguments are invalid, 0 is returned.
    pub fn dpmm_from_scale_and_width(
        self,
        scale: f32,
        xdim: f32,
        filetype: TargetFiletype,
    ) -> Result<f32, UnitConversionError> {
        if xdim <= 0.0 || xdim > 10.0 {
            return Err(UnitConversionError::InvalidXDimension(xdim));
        }
        // X-dim bounds are smaller than dots_per_mm bounds, so
        // UnitConversionError::InvalidDpmm will never be returned
        self.xdim_from_scale_and_dpmm(scale, xdim, filetype)
    }

    pub fn capabilities(&self) -> CapabilityFlags {
        let capabilities = unsafe{
            zint_sys::ZBarcode_Cap(*self as i32, std::ffi::c_uint::MAX)
        };
        CapabilityFlags::from_bits_retain(capabilities)
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum TargetFiletype {
    #[default]
    Raster,
    Vector,
    /// EMF vector format is scaled 20x more to compensate for limitations.
    EMF,
}

impl TargetFiletype {
    pub fn from_ext(ext: &str) -> Self {
        match ext {
            "emf" => Self::EMF,
            "svg" | "ps" => Self::Vector,
            _ => Self::Raster,
        }
    }

    pub fn as_c_str(&self) -> &'static std::ffi::CStr {
        match self {
            TargetFiletype::Raster => c"GIF",
            TargetFiletype::Vector => c"SVG",
            TargetFiletype::EMF => c"EMF",
        }
    }
}

#[derive(Debug)]
pub enum UnitConversionError {
    InvalidScale(f32),
    InvalidDpmm(f32),
    InvalidXDimension(f32),
}

impl Display for UnitConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn dpc(dpmm: f32) -> f32 {
            dpmm * 25.4
        }
        fn mm_to_in(mm: f32) -> f32 {
            mm * 5.0 / 127.0
        }
        fn numeric_fault(value: f32) -> &'static str {
            if value < 0.0 {
                "negative"
            } else if value == 0.0 {
                "zero"
            } else {
                "too large"
            }
        }
        match self {
            UnitConversionError::InvalidScale(scale) => write!(
                f,
                "provided scale factor ({scale}) is {}",
                numeric_fault(*scale)
            ),
            UnitConversionError::InvalidDpmm(dpmm) => write!(
                f,
                "provided dots/mm value ({dpmm}dpmm; {}dpi) is {}",
                dpc(*dpmm),
                numeric_fault(*dpmm)
            ),
            UnitConversionError::InvalidXDimension(width) => write!(
                f,
                "provided width ({width}mm; {}\") is {}",
                mm_to_in(*width),
                numeric_fault(*width)
            ),
        }
    }
}
impl std::error::Error for UnitConversionError {}

pub trait ConfigureSymbolOptions<'o> {
    fn configure(&self, result: &mut GenericOptions<'o>) -> Result<(), SymbolOptionError>;
}

fn require_range_inclusive<T: PartialOrd + Display>(
    name: &'static str,
    value: T,
    min: T,
    max: T,
) -> Result<T, SymbolOptionError> {
    if value < min || value > max {
        Err(SymbolOptionError {
            option_name: name,
            value: value.to_string(),
            reason: format!("must be a value between {min} and {max}"),
        })
    } else {
        Ok(value)
    }
}

impl Display for Symbology {
    #[allow(deprecated)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pretty_name = match self {
            Self::Code11 => "Code 11",
            Self::C25Standard => "2 of 5 Standard (Matrix)",
            Self::C25Inter => "2 of 5 Interleaved",
            Self::C25IATA => "2 of 5 IATA",
            Self::C25Logic => "2 of 5 Data Logic",
            Self::C25Ind => "2 of 5 Industrial",
            Self::Code39 => "Code 39",
            Self::ExCode39 => "Extended Code 39",
            Self::EAN8 => "European Article Number GTIN-8",
            Self::EAN2 => "EAN/UPC 2-digit",
            Self::EAN5 => "EAN/UPC 5-digit",
            Self::EANX => "European Article Number",
            Self::EANXChk => "European Article Number with digit check",
            Self::EAN13 => "European Article Number GTIN-13",
            Self::GS1128 => "GS1-128",
            Self::Codabar => "Codabar",
            Self::Code128 => "Code 128",
            Self::DPLEIT => "Deutsche Post Leitcode",
            Self::DPIDENT => "Deutsche Post Identcode",
            Self::Code16k => "Code 16k",
            Self::Code49 => "Code 49",
            Self::Code93 => "Code 93",
            Self::Flat => "Flattermarken",
            Self::DBarOmn => "GS1 DataBar Omnidirectional",
            Self::DBarLtd => "GS1 DataBar Limited",
            Self::DBarExp => "GS1 DataBar Expanded",
            Self::Telepen => "Telepen Alpha",
            Self::UPCA => "UPC-A",
            Self::UPCAChk => "UPC-A including check digit",
            Self::UPCE => "UPC-E",
            Self::UPCEChk => "UPC-E including check digit",
            Self::Postnet => "USPS POSTNET",
            Self::MSIPlessey => "MSI Plessey",
            Self::FIM => "Facing Identification Mark",
            Self::LOGMARS => "LOGMARS",
            Self::Pharma => "Pharmacode One-Track",
            Self::PZN => "Pharmazentralnummer",
            Self::PharmaTwo => "Pharmacode Two-Track",
            Self::CEPNet => "Brazilian CEPNet Postal Code",
            Self::PDF417 => "PDF417",
            Self::PDF417Comp => "Compact/truncated PDF417",
            Self::MaxiCode => "MaxiCode",
            Self::QRCode => "QR Code",
            Self::Code128AB => "Code 128 (Suppress Code Set C)",
            Self::AusPost => "Australia Post Standard Customer",
            Self::AusReply => "Australia Post Reply Paid",
            Self::AusRoute => "Australia Post Routing",
            Self::AusRedirect => "Australia Post Redirection",
            Self::ISBNX => "ISBN",
            Self::RM4SCC => "Royal Mail 4-State Customer Code",
            Self::DataMatrix => "Data Matrix (ECC200)",
            Self::EAN14 => "EAN-14",
            Self::VIN => "Vehicle Identification Number",
            Self::CodablockF => "Codablock-F",
            Self::NVE18 => "NVE-18 (SSCC-18)",
            Self::JapanPost => "Japanese Postal Code",
            Self::KoreaPost => "Korea Post",
            Self::DBarStk => "GS1 DataBar Stacked",
            Self::DBarOmnStk => "GS1 DataBar Stacked Omnidirectional",
            Self::DBarExpStk => "GS1 DataBar Expanded Stacked",
            Self::Planet => "USPS PLANET",
            Self::MicroPDF417 => "MicroPDF417",
            Self::USPSIMail => "USPS Intelligent Mail (OneCode)",
            Self::Plessey => "UK Plessey",
            Self::TelepenNum => "Telepen Numeric",
            Self::ITF14 => "ITF-14",
            Self::KIX => "Dutch Post KIX Code",
            Self::Aztec => "Aztec Code",
            Self::DAFT => "DAFT Code",
            Self::DPD => "DPD Code",
            Self::MicroQR => "Micro QR Code",
            Self::HIBC128 => "HIBC Code 128",
            Self::HIBC39 => "HIBC Code 39",
            Self::HIBCDM => "HIBC Data Matrix",
            Self::HIBCQR => "HIBC QR Code",
            Self::HIBCPDF => "HIBC PDF417",
            Self::HIBCMicroPDF => "HIBC MicroPDF417",
            Self::HIBCCodablockF => "HIBC Codablock-F",
            Self::HIBCAztec => "HIBC Aztec Code",
            Self::DotCode => "DotCode",
            Self::HanXin => "Han Xin (Chinese Sensible) Code",
            Self::Mailmark2D => "Royal Mail 2D Mailmark (CMDM) (Data Matrix)",
            Self::UPUS10 => "Universal Postal Union S10",
            Self::Mailmark4S => "Royal Mail 4-State Mailmark",
            Self::AzRune => "Aztec Runes",
            Self::Code32 => "Code 32",
            Self::EANXCC => "Legacy",
            Self::GS1128CC => "GS1-128 Composite",
            Self::DBarOmnCC => "GS1 DataBar Omnidirectional Composite",
            Self::DBarLtdCC => "GS1 DataBar Limited Composite",
            Self::DBarExpCC => "GS1 DataBar Expanded Composite",
            Self::UPCACC => "UPC-A Composite",
            Self::UPCECC => "UPC-E Composite",
            Self::DBarStkCC => "GS1 DataBar Stacked Composite",
            Self::DBarOmnStkCC => "GS1 DataBar Stacked Omnidirectional Composite",
            Self::DBarExpStkCC => "GS1 DataBar Expanded Stacked Composite",
            Self::Channel => "Channel Code",
            Self::CodeOne => "Code One",
            Self::GridMatrix => "Grid Matrix",
            Self::UPNQR => "Univerzalnega Plačilnega Naloga QR",
            Self::Ultra => "Ultracode",
            Self::RMQR => "Rectangular Micro QR Code",
            Self::BC412 => "IBM BC412 (SEMI T1-95)",
            Self::DXFilmEdge => "DX Film Edge Barcode on 35mm and APS films",
            Self::EAN8CC => "EAN-8 Composite",
            Self::EAN13CC => "EAN-13 Composite",
        };
        f.write_str(pretty_name)
    }
}

#[derive(Debug)]
pub struct SymbolOptionError {
    pub option_name: &'static str,
    pub value: String,
    pub reason: String,
}

impl Display for SymbolOptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let SymbolOptionError {
            option_name,
            value,
            reason,
        } = self;
        write!(f, "invalid `{option_name}` value ({value}); {reason}")
    }
}
impl std::error::Error for SymbolOptionError {}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    #[test]
    fn verify_coverage() {
        let root = {
            let current = std::env::current_exe().unwrap();
            let mut current = Some(current.as_path());
            for _ in 0..4 {
                current = current.and_then(|p| p.parent());
            }
            current.unwrap().to_path_buf()
        };

        let source = std::fs::read_to_string(root.join("zint-wasm-sys/zint/backend/zint.h"))
            .expect("can't find zint.h");
        let source: HashSet<String> = source
            .lines()
            .filter(|it| it.starts_with("#define BARCODE_"))
            .filter_map(|it| {
                if it.contains("Legacy") {
                    None
                } else {
                    let start = &it[8..];
                    let name = start.chars().take_while(|it| !it.is_whitespace()).collect();
                    Some(name)
                }
            })
            .take_while(|it| it != "BARCODE_LAST")
            .collect();

        let target = std::fs::read_to_string(root.join("zint-wasm-rs/src/options/symbology.rs"))
            .expect("can't find symbology.rs");
        let target = target
            .split_once("pub enum Symbology {\n")
            .expect("unexpected symbology.rs content")
            .1;
        let mut target: HashSet<String> = target
            .lines()
            .take_while(|it| it.trim() != "}")
            .filter_map(|it| {
                let it = it.trim();
                if it.starts_with("#[") || it.starts_with("//") {
                    None
                } else if it.ends_with(" as i32,") {
                    let it = it.split_once(" = ").expect("unexpected syntax").1;
                    let it = it.chars().take_while(|it| !it.is_whitespace()).collect();
                    Some(it)
                } else {
                    None
                }
            })
            .collect();

        let mut missing = Vec::new();
        for s in source {
            if !target.remove(&s) {
                missing.push(s);
            }
        }

        if !missing.is_empty() {
            let missing = missing.join(", ");
            panic!("Missing Symbology enum variants for: {missing}")
        }
    }
}
