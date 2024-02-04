use serde::Deserialize;
use zint_wasm_sys::*;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(tag = "symbology", rename_all = "PascalCase")]
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
    BC412 = BARCODE_LAST as i32,
}
