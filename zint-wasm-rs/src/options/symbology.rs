use serde::Deserialize;
use std::{ffi::CString, fmt::Display};
use zint_rs_macros::symbol_data;
use zint_sys::*;
use crate::options::GenericOptions;
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
// it's meant for internal use only and functionality covers only the needs if
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
            options: C25Options,
        },
        /// 2 of 5 IATA
        C25IATA = {
            raw: BARCODE_C25IATA,
            options: C25Options,
        },
        /// 2 of 5 Data Logic
        C25Logic = {
            raw: BARCODE_C25LOGIC,
            options: C25Options,
        },
        /// 2 of 5 Industrial
        C25Ind = {
            raw: BARCODE_C25IND,
            options: C25Options,
        },
        /// Code 39
        Code39 = {
            raw: BARCODE_CODE39,
        },
        /// Extended Code 39
        ExCode39 = {
            raw: BARCODE_EXCODE39,
        },
        /// Variable length EAN variant
        #[deprecated(note = "will be removed; use specialized EAN code variants")]
        EANX = {
            raw: BARCODE_EANX,
            options: UPCEOptions,
        },
        /// Variable length EAN variant with digit check
        #[deprecated(note = "will be removed; use specialized EAN CHK code variants")]
        EANXChk = {
            raw: BARCODE_EANX_CHK,
            options: UPCEOptions,
        },
        /// Variable length composite EAN variant
        #[deprecated(note = "will be removed; use specialized EAN CC code variants")]
        EANXCC = {
            raw: BARCODE_EANX_CC,
            options: UPCEOptions,
        },
        /// EAN/UPC 2-digit
        /// 
        /// This symbology is almost never used as standalone and is generally
        /// appended to other one-dimensional barcodes.
        EAN2 = {
            raw: BARCODE_EAN_2ADDON,
            alias: "EAN2Addon",
            options: UPCEOptions,
        },
        /// EAN/UPC 5-digit
        /// 
        /// This symbology is almost never used as standalone and is generally
        /// appended to other one-dimensional barcodes.
        EAN5 = {
            raw: BARCODE_EAN_5ADDON,
            alias: "EAN5Addon",
            options: UPCEOptions,
        },
        /// EAN-8 (European Article Number) GTIN-8
        /// 
        /// In addition EAN-2 and EAN-5 add-on symbols can be added by using the
        /// '+' character as a separator after EAN-8 data.
        EAN8 = {
            raw: BARCODE_EAN8,
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
            alias: "GTIN14",
            options: UPCEOptions,
        },
        /// EAN-8 Composite
        EAN8CC = {
            raw: BARCODE_EAN8_CC,
            options: UPCEOptions,
        },
        /// EAN-13 Composite
        EAN13CC = {
            raw: BARCODE_EAN13_CC,
            options: UPCEOptions,
        },
        /// GS1-128
        GS1128 = {
            raw: BARCODE_GS1_128,
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
        },
        /// Deutsche Post Leitcode is based on Interleaved Code 2 of 5 and is
        /// used by Deutsche Post for mailing purposes. Leitcode requires a
        /// 13-digit numerical input and includes a check digit.
        DPLEIT = {
            raw: BARCODE_DPLEIT,
        },
        /// Deutsche Post Identcode is based on Interleaved Code 2 of 5 and is
        /// used by Deutsche Post for mailing purposes. Identcode requires
        /// 11-digit numerical input and includes a check digit.
        DPIDENT = {
            raw: BARCODE_DPIDENT,
        },
        /// Code 16k
        Code16k = {
            raw: BARCODE_CODE16K,
        },
        /// Code 49
        Code49 = {
            raw: BARCODE_CODE49,
        },
        /// Code 93
        Code93 = {
            raw: BARCODE_CODE93,
        },
        /// Flattermarken
        Flat = {
            raw: BARCODE_FLAT,
        },
        /// GS1 DataBar Omnidirectional
        DBarOmn = {
            raw: BARCODE_DBAR_OMN,
            alias: "RSS14"
        },
        /// GS1 DataBar Limited
        DBarLtd = {
            raw: BARCODE_DBAR_LTD,
        },
        /// GS1 DataBar Expanded
        DBarExp = {
            raw: BARCODE_DBAR_EXP,
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
            options: {
                /// Gap between the main symbol and an add-on in multiples of the X-dimension.
                addon_gap: usize = 9,
                /// Height in X-dimensions that the guard bars descend below the main bars.
                guard_descent: f32 = 5.0,
            },
            apply_options: |result, options| {
                result.option_2 = Some(require_range_inclusive("addon_gap", options.addon_gap as i32, 9, 12)?);
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
            options: UPCAOptions,
        },
        /// UPC-E
        UPCE = {
            raw: BARCODE_UPCE,
            options: {
                /// Gap between the main symbol and an add-on in multiples of the X-dimension.
                addon_gap: usize = 7,
                /// Height in X-dimensions that the guard bars descend below the main bars.
                guard_descent: f32 = 5.0,
            },
            apply_options: |result, options| {
                result.option_2 = Some(require_range_inclusive("addon_gap", options.addon_gap as i32, 7, 12)?);
                result.guard_descent =
                    require_range_inclusive("guard_descent", options.guard_descent, 0.0, 20.0)?;
                Ok(())
            }
        },
        /// UPC-E including check digit
        UPCEChk = {
            raw: BARCODE_UPCE_CHK,
            options: UPCEOptions,
        },
        /// USPS (U.S. Postal Service) POSTNET
        Postnet = {
            raw: BARCODE_POSTNET,
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
                    let mut value = options.check_digits.into();
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
        /// Pharmacode One-Track
        Pharma = {
            raw: BARCODE_PHARMA,
        },
        /// Pharmazentralnummer
        PZN = {
            raw: BARCODE_PZN,
        },
        /// Pharmacode Two-Track
        PharmaTwo = {
            raw: BARCODE_PHARMA_TWO,
        },
        /// Brazilian CEPNet Postal Code
        CEPNet = {
            raw: BARCODE_CEPNET,
        },
        /// PDF417
        PDF417 = {
            raw: BARCODE_PDF417,
        },
        /// Compact PDF417 (Truncated PDF417)
        PDF417Comp = {
            raw: BARCODE_PDF417COMP,
            alias: "PDF417Trunc",
        },
        /// MaxiCode
        /// 
        /// This symbology is designed for the identification of parcels.
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
                    result.option_1 = Some(mode);
                } else if let Some(scm_prefix) = options.scm_prefix {
                    let scm_prefix = require_range_inclusive("scm_prefix", scm_prefix, 0, 99)?;
                    // Will still return invalid option error if data isn't
                    // using an ASCII-compatible ECI. Can't check that as it
                    // isn't known yet.
                    result.option_2 = Some((scm_prefix + 1) as i32);
                }
                Ok(())
            }
        },
        /// QR Code
        QRCode = {
            raw: BARCODE_QRCODE,
        },
        /// Code 128 (Suppress Code Set C)
        Code128AB = {
            raw: BARCODE_CODE128AB,
            alias: "CODE128B"
        },
        /// Australia Post Standard Customer
        AusPost = {
            raw: BARCODE_AUSPOST,
        },
        /// Australia Post Reply Paid
        AusReply = {
            raw: BARCODE_AUSREPLY,
        },
        /// Australia Post Routing
        AusRoute = {
            raw: BARCODE_AUSROUTE,
        },
        /// Australia Post Redirection
        AusRedirect = {
            raw: BARCODE_AUSREDIRECT,
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
            options: UPCEOptions,
        },
        /// Royal Mail 4-State Customer Code
        RM4SCC = {
            raw: BARCODE_RM4SCC,
        },
        /// Data Matrix (ECC200)
        DataMatrix = {
            raw: BARCODE_DATAMATRIX,
        },
        /// Vehicle Identification Number
        VIN = {
            raw: BARCODE_VIN,
        },
        /// Codablock-F
        CodablockF = {
            raw: BARCODE_CODABLOCKF,
        },
        /// NVE-18 (SSCC-18)
        NVE18 = {
            raw: BARCODE_NVE18,
        },
        /// Japanese Postal Code
        JapanPost = {
            raw: BARCODE_JAPANPOST,
        },
        /// Korea Post
        KoreaPost = {
            raw: BARCODE_KOREAPOST,
        },
        /// GS1 DataBar Stacked
        DBarStk = {
            raw: BARCODE_DBAR_STK,
            alias: "RSS14Stk"
        },
        /// GS1 DataBar Stacked Omnidirectional
        DBarOmnStk = {
            raw: BARCODE_DBAR_OMNSTK,
            alias: "RSS14StackOmni"
        },
        /// GS1 DataBar Expanded Stacked
        #[serde(alias = "RSSExpStack")]
        DBarExpStk = {
            raw: BARCODE_DBAR_EXPSTK,
        },
        /// USPS PLANET
        Planet = {
            raw: BARCODE_PLANET,
        },
        /// MicroPDF417
        MicroPDF417 = {
            raw: BARCODE_MICROPDF417,
        },
        /// USPS Intelligent Mail (OneCode)
        USPSIMail = {
            raw: BARCODE_USPS_IMAIL,
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
        },
        /// Dutch Post KIX Code
        KIX = {
            raw: BARCODE_KIX,
        },
        /// Aztec Code
        Aztec = {
            raw: BARCODE_AZTEC,
        },
        /// DAFT Code
        DAFT = {
            raw: BARCODE_DAFT,
        },
        /// DPD Code
        DPD = {
            raw: BARCODE_DPD,
        },
        /// Micro QR Code
        MicroQR = {
            raw: BARCODE_MICROQR,
        },

        // Tbarcode 9 codes
        /// HIBC (Health Industry Barcode) Code 128
        HIBC128 = {
            raw: BARCODE_HIBC_128,
        },
        /// HIBC Code 39
        HIBC39 = {
            raw: BARCODE_HIBC_39,
        },
        /// HIBC Data Matrix
        HIBCDM = {
            raw: BARCODE_HIBC_DM,
        },
        /// HIBC QR Code
        HIBCQR = {
            raw: BARCODE_HIBC_QR,
        },
        /// HIBC PDF417
        HIBCPDF = {
            raw: BARCODE_HIBC_PDF,
        },
        /// HIBC MicroPDF417
        HIBCMicroPDF = {
            raw: BARCODE_HIBC_MICPDF,
            alias: "HIBCMicPDF"
        },
        /// HIBC Codablock-F
        HIBCCodablockF = {
            raw: BARCODE_HIBC_BLOCKF,
            alias: "HIBCBlockF"
        },
        /// HIBC Aztec Code
        HIBCAztec = {
            raw: BARCODE_HIBC_AZTEC,
        },

        // Tbarcode 10 codes
        /// DotCode
        DotCode = {
            raw: BARCODE_DOTCODE,
        },
        /// Han Xin (Chinese Sensible) Code
        HanXin = {
            raw: BARCODE_HANXIN,
        },

        // Tbarcode 11 codes
        /// Royal Mail 2D Mailmark (CMDM) (Data Matrix)
        Mailmark2D = {
            raw: BARCODE_MAILMARK_2D,
        },
        /// Universal Postal Union S10
        UPUS10 = {
            raw: BARCODE_UPU_S10,
        },
        /// Royal Mail 4-State Mailmark
        Mailmark4S = {
            raw: BARCODE_MAILMARK_4S,
            alias: "Mailmark"
        },

        // Zint specific codes
        /// Aztec Runes
        AzRune = {
            raw: BARCODE_AZRUNE,
        },
        /// Code 32
        Code32 = {
            raw: BARCODE_CODE32,
        },
        /// GS1-128 Composite
        GS1128CC = {
            raw: BARCODE_GS1_128_CC,
            alias: "EAN128CC",
        },
        /// GS1 DataBar Omnidirectional Composite
        DBarOmnCC = {
            raw: BARCODE_DBAR_OMN_CC,
            alias: "RSS14CC",
        },
        /// GS1 DataBar Limited Composite
        DBarLtdCC = {
            raw: BARCODE_DBAR_LTD_CC,
            alias: "RSSLtdCC",
        },
        /// GS1 DataBar Expanded Composite
        DBarExpCC = {
            raw: BARCODE_DBAR_EXP_CC,
            alias: "RSSExpCC",
        },
        /// UPC-A Composite
        UPCACC = {
            raw: BARCODE_UPCA_CC,
        },
        /// UPC-E Composite
        UPCECC = {
            raw: BARCODE_UPCE_CC,
        },
        /// GS1 DataBar Stacked Composite
        DBarStkCC = {
            raw: BARCODE_DBAR_STK_CC,
            alias: "RSS14StackCC",
        },
        /// GS1 DataBar Stacked Omnidirectional Composite
        DBarOmnStkCC = {
            raw: BARCODE_DBAR_OMNSTK_CC,
            alias: "RSS14OmniCC",
        },
        /// GS1 DataBar Expanded Stacked Composite
        DBarExpStkCC = {
            raw: BARCODE_DBAR_EXPSTK_CC,
            alias: "RSSExpStackCC",
        },
        /// Channel Code
        Channel = {
            raw: BARCODE_CHANNEL,
        },
        /// Code One
        CodeOne = {
            raw: BARCODE_CODEONE,
        },
        /// Grid Matrix
        GridMatrix = {
            raw: BARCODE_GRIDMATRIX,
        },
        /// UPNQR (Univerzalnega Plačilnega Naloga QR)
        UPNQR = {
            raw: BARCODE_UPNQR,
        },
        /// Ultracode
        Ultra = {
            raw: BARCODE_ULTRA,
        },
        /// Rectangular Micro QR Code (rMQR)
        RMQR = {
            raw: BARCODE_RMQR,
        },
        /// IBM BC412 (SEMI T1-95)
        BC412 = {
            raw: BARCODE_BC412,
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
            unsafe { ZBarcode_BarcodeName(self as i32, read_buffer.as_mut_ptr() as *mut i8) };
        if result == 1 {
            panic!("Symbology value is invalid");
        }
        // SAFETY: zint always insterts a nul byte at the end
        let read_buffer = unsafe { CString::from_vec_with_nul_unchecked(read_buffer) };
        let result = unsafe { read_buffer.to_str().unwrap_unchecked() };
        return result.to_string();
    }

    /// Returns default width in mm for this symbology.
    pub fn default_width(self) -> f32 {
        unsafe { ZBarcode_Default_Xdim(self as i32) }
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
            ZBarcode_Scale_From_XdimDp(self as i32, xdim, dots_per_mm, filetype.as_c_str().as_ptr())
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
                self as i32,
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

    pub(crate) fn supports_eci(&self) -> bool {
        // Values are copied from `static int supports_eci(const int symbology)`
        // located in zint/backend/library.c
        matches!(
            self,
            Self::Aztec
                | Self::DataMatrix
                | Self::MaxiCode
                | Self::MicroPDF417
                | Self::PDF417
                | Self::PDF417Comp
                | Self::QRCode
                | Self::DotCode
                | Self::CodeOne
                | Self::GridMatrix
                | Self::HanXin
                | Self::Ultra
                | Self::RMQR
        )
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
