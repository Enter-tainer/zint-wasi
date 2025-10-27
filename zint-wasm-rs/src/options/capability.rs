use zint_sys::*;

bitflags::bitflags! {
    /// Capability flags (ZBarcode_Cap() `cap_flag`)
    #[derive(Debug, Copy, Clone)]
    pub struct CapabilityFlags: std::ffi::c_uint {
        /// Prints Human Readable Text?
        const HRT = ZINT_CAP_HRT;
        /// Is stackable?
        const STACKABLE = ZINT_CAP_STACKABLE;
        /// Is EAN/UPC?
        const EAN_UPC = ZINT_CAP_EANUPC;
        /// Legacy
        const EXTENDABLE = ZINT_CAP_EXTENDABLE;
        /// Can have composite data?
        const COMPOSITE = ZINT_CAP_COMPOSITE;
        /// Supports Extended Channel Interpretations?
        const ECI = ZINT_CAP_ECI;
        /// Supports GS1 data?
        const GS1 = ZINT_CAP_GS1;
        /// Can be output as dots?
        const DOTTY = ZINT_CAP_DOTTY;
        /// Has default quiet zones?
        const QUIET_ZONES = ZINT_CAP_QUIET_ZONES;
        /// Has fixed width-to-height (aspect) ratio?
        const FIXED_RATIO = ZINT_CAP_FIXED_RATIO;
        /// Supports Reader Initialisation?
        const READER_INIT = ZINT_CAP_READER_INIT;
        /// Supports full-multibyte option?
        const FULL_MULTIBYTE = ZINT_CAP_FULL_MULTIBYTE;
        /// Is mask selectable?
        const MASK = ZINT_CAP_MASK;
        /// Supports Structured Append?
        const STRUCTURED_APPEND = ZINT_CAP_STRUCTAPP;
        /// Has compliant height?
        const COMPLIANT_HEIGHT = ZINT_CAP_COMPLIANT_HEIGHT;
        /// Has row separators that can be set?
        /// 
        /// Includes stacked symbologies and stackable linear symbologies.
        const BINDABLE = ZINT_CAP_BINDABLE;
    }
}
