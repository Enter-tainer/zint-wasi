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
    use std::{collections::HashMap, ffi::CStr, os::raw::c_char};
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

    /// The names these symbologies used to carry, kept so that documents
    /// written against older zint releases keep working.
    #[test]
    fn the_former_names_of_renamed_symbologies_are_accepted() {
        for (alias, expected) in [
            ("C25Matrix", Symbology::C25Standard),
            ("EAN128", Symbology::GS1128),
            ("RSS14", Symbology::DBarOmn),
            ("RSSLtd", Symbology::DBarLtd),
            ("RSSExpStack", Symbology::DBarExpStk),
            ("OneCode", Symbology::USPSIMail),
            ("PDF417Trunc", Symbology::PDF417Comp),
            ("Code128B", Symbology::Code128AB),
            ("Mailmark", Symbology::Mailmark4S),
            ("RSS14CC", Symbology::DBarOmnCC),
        ] {
            let parsed: Options = from_cbor(cbor!({ "symbology" => alias }).unwrap())
                .unwrap_or_else(|error| panic!("{alias} is not accepted: {error}"));

            assert_eq!(parsed.symbology as i32, expected as i32, "{alias}");
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
