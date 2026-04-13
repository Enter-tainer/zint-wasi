use crate::segment::Segment;
#[derive(Debug, Clone, Copy)]
pub enum ModuloScheme {
    Luhn,
    IBM,
    NCR,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum MSIPlesseyCheckDigits {
    #[default]
    None,
    One(ModuloScheme),
    /// Second check digit always uses [Luhn][ModuloScheme::Luhn] modulo scheme.
    Two(ModuloScheme),
}

impl From<MSIPlesseyCheckDigits> for std::ffi::c_int {
    fn from(val: MSIPlesseyCheckDigits) -> Self {
        match val {
            MSIPlesseyCheckDigits::None => 0,
            MSIPlesseyCheckDigits::One(modulo_scheme) => match modulo_scheme {
                ModuloScheme::Luhn => 1,
                ModuloScheme::IBM => 3,
                ModuloScheme::NCR => 5,
            },
            MSIPlesseyCheckDigits::Two(modulo_scheme) => match modulo_scheme {
                ModuloScheme::Luhn => 2,
                ModuloScheme::IBM => 4,
                ModuloScheme::NCR => 6,
            },
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum MaxiCodeMode<'o> {
    /// Structured Carrier Message with numeric or alphanumeric postal code.
    ///
    /// When this variant is used, zint will pick mode 2 or mode 3 automatically
    /// based on provided `Segment` data. Mode 2 is numeric only, mode 3 is
    /// alphanumeric.
    ///
    /// Provided `Segment` must use the default MaxiCode character set ([ISO
    /// 8859-1 (Latin-1)][crate::segment::ECI::ISO_8859_1]), and must not
    /// contain the backslash escape character to change the ECI character set.
    ///
    /// This mode is designed for use in the transport industry, encoding the
    /// postal code, country code, and service class with the postal code that
    /// is alphanumeric.
    ///
    /// Format of the primary message is expected by zint to consist of
    /// following segments:
    /// - **1-9 decimal digits:** postal code data which can consist of up to 9
    ///   digits (mode 2) or up to 6 alphanumeric characters (mode 3).
    ///   
    ///   Remaining 3 unused characters can be filled with the space character
    ///   ('\x20') or omitted.
    /// - **3 decimal digits:** three-digit country code according to ISO
    ///   3166-1.
    /// - **3 decimal digits:** three-digit service code, which depends on
    ///   parcel courier.
    ///
    /// The primary message portion of the MaxiCode symbol uses Enhanced Error
    /// Correction (EEC) and the secondary message portion of the MaxiCode
    /// symbol uses Standard Error Correction (SEC).
    StructuredCarrierMessage(Segment<'o>),
    /// Standard Symbol.
    ///
    /// The symbol employs Enhanced Error Correction (EEC) for the Primary
    /// Message and Standard Error Correction (SEC) for the Secondary Message.
    ///
    /// The first nine codewords of data provided to Zint are placed in the
    /// Primary Message and the rest of the codewords are placed in the
    /// Secondary Message. Zint doesn't
    ///
    /// This mode provides for a total of 93 codewords for data. If the bar code
    /// data consists of only characters from MaxiCode Code Set A, the number of
    /// codewords matches the number of bar code data characters. However, if
    /// the bar code data contains other characters, the number of codewords is
    /// greater than the number of bar code data characters due to the overhead
    /// of switching to and from the different code sets.
    ///
    /// The Code Set A consists of the byte values `\x0D`, `\x1C`-`\x1E`,
    /// `\x20`, `\x22`-`\x3A`, and `\x41`-`\x5A`.
    ///
    /// This variant corresponds to MaxiCode mode 4.
    #[default]
    StandardCorrection,
    /// Full ECC Symbol.
    ///
    /// The symbol employs EEC for the Primary Message and EEC for the Secondary
    /// Message.
    ///
    /// Primary and Secondary Message segmentation is same as in
    /// [`StandardCorrection`] (mode 4).
    ///
    /// This mode provides for a total of 77 codewords for data. With same
    /// encoding characteristics as in [`StandardCorrection`].
    ///
    /// This variant corresponds to MaxiCode mode 5.
    ///
    /// [`StandardCorrection`]: MaxiCodeMode::StandardCorrection
    EnhancedCorrection,
    /// Reader Program, SEC.
    ///
    /// The symbol employs EEC for the Primary Message and SEC for the Secondary
    /// Message. The data in the symbol is used to program the bar code reader
    /// system.
    ///
    /// Codeword limitations are same as in
    /// [`StandardCorrection`][MaxiCodeMode::StandardCorrection] (mode 4).
    ///
    /// This variant corresponds to MaxiCode mode 6.
    Programming,
}

impl<'o> MaxiCodeMode<'o> {
    pub(crate) fn mode(&self) -> Option<std::ffi::c_int> {
        match self {
            MaxiCodeMode::StructuredCarrierMessage(_) => None, /* automatically detected by zint */
            MaxiCodeMode::StandardCorrection => Some(4),
            MaxiCodeMode::EnhancedCorrection => Some(5),
            MaxiCodeMode::Programming => Some(6),
        }
    }

    pub(crate) fn primary_message(&self) -> Option<Segment<'o>> {
        if let MaxiCodeMode::StructuredCarrierMessage(result) = self {
            Some(*result)
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UltracodeErrorCorrection {
    /// Provides 0% error correction (only error detection).
    EC0 = 1,
    /// Provides ~5% error correction.
    EC1 = 2,
    /// Provides ~9% error correction.
    #[default]
    EC2 = 3,
    /// Provides ~17% error correction.
    EC3 = 4,
    /// Provides ~25% error correction.
    EC4 = 5,
    /// Provides ~33% error correction.
    EC5 = 6,
}

impl UltracodeErrorCorrection {
    /// Produces an error correction level based on allowed ratio of symbol
    /// coverage.
    /// 
    /// Result will be rounded down to the largest level that uses less or equal
    /// ratio of the symbol area. E.g. a value of `0.32` will be rounded down to
    /// `EC4`, because `EC5` would on average occupy more than 32% of symbol
    /// area.
    pub fn from_ratio(ratio: f32) -> Self {
        match (ratio * 100.) as u32 {
            33.. => Self::EC5,
            25.. => Self::EC4,
            17.. => Self::EC3,
            9.. => Self::EC2,
            5.. => Self::EC1,
            _ => Self::EC0
        }
    }
}

/// Specifies
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UltracodeRevision(pub(crate) u8);

impl UltracodeRevision {
    /// Base (default) Ultracode revision.
    pub const REVISION_1: UltracodeRevision = UltracodeRevision(0);
    /// Swaps and inverts the _Diagnostic Colour Calibration Upper_ (DCCU) and
    /// _Diagnostic Colour Calibration Lower_ (DCCL) tiles.
    pub const REVISION_2: UltracodeRevision = UltracodeRevision(1);
}

/// Specifies amount of error correction data to include in the QR code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum QRErrorCorrection {
    /// Approximately 20% of the symbol is used for error correction data, and
    /// symbol has ~7% error recovery capacity.
    L,
    /// Approximately 37% of the symbol is used for error correction data, and
    /// symbol has ~15% error recovery capacity.
    M,
    /// Approximately 55% of the symbol is used for error correction data, and
    /// symbol has ~25% error recovery capacity.
    Q,
    /// Approximately 65% of the symbol is used for error correction data, and
    /// symbol has ~30% error recovery capacity.
    H,
}

/// Size of the QR code.
/// 
/// The maximum capacity of a QR Code symbol (version 40) is 7089 numeric
/// digits, 4296 alphanumeric characters or 2953 bytes of data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct QRSize(u8);

impl QRSize {
    const SIZES: &[u8] = &[
        21,25,29,33,37,41,45,49,53,57,61,65,69,73,
        77,81,85,89,93,97,101,105,109,113,117,121,125,129,
        133,137,141,145,149,153,157,161,165,169,173,177,
    ];

    pub fn version(number: usize) -> Self {
        Self(number.min(Self::SIZES.len()) as u8)
    }

    pub fn at_least_size(size: usize) -> Option<Self> {
        if size > 177 {
            return None;
        }
        let found = Self::SIZES.iter().enumerate().find_map(|(i, it)| {
            if *it >= size as u8 {
                Some(i as u8)
            } else {
                None
            }
        })?;
        Some(Self(found))
    }

    pub fn at_most_size(size: usize) -> Option<Self> {
        if size < 21 {
            return None;
        }
        let found = Self::SIZES.iter().enumerate().rev().find_map(|(i, it)| {
            if *it <= size as u8 {
                Some(i as u8)
            } else {
                None
            }
        })?;
        Some(Self(found))
    }

    pub fn size(&self) -> usize {
        Self::SIZES[self.0 as usize] as usize
    }
}

impl From<QRSize> for u8 {
    fn from(size: QRSize) -> Self {
        size.0
    }
}

/// QR mask used to minimize unwanted patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QRMask {
    #[doc = include_str!("../../../assets/masks/mask000.svg")]
    ///
    /// Applies a mask where modules alternate between dark and light every
    /// other module in both rows and columns.
    ///
    /// Formula: `(i + j) % 2 = 0`
    Mask0 = 0b000,
    #[doc = include_str!("../../../assets/masks/mask001.svg")]
    ///
    /// Modules alternate every other column.
    ///
    /// Formula: `i % 2 = 0`
    Mask1 = 0b001,
    #[doc = include_str!("../../../assets/masks/mask010.svg")]
    ///
    /// Alternates every other row.
    ///
    /// Formula: `j % 3 = 0`
    Mask2 = 0b010,
    #[doc = include_str!("../../../assets/masks/mask011.svg")]
    ///
    /// Alternates based on a combination of both rows and columns but with a
    /// more complex formula.
    ///
    /// Formula: `(i + j) % 3 = 0`
    Mask3 = 0b011,
    #[doc = include_str!("../../../assets/masks/mask100.svg")]
    ///
    /// Modules change depending on their diagonal position.
    ///
    /// Formula: `(i/2 + j/3) % 2 = 0`
    Mask4 = 0b100,
    #[doc = include_str!("../../../assets/masks/mask101.svg")]
    ///
    /// A specific rule based on the sum of the row and column indices.
    ///
    /// Formula: `(i*j) % 2 + (i*j) % 3 = 0`
    Mask5 = 0b101,
    #[doc = include_str!("../../../assets/masks/mask110.svg")]
    ///
    /// Modules change based on the parity of the row and column.
    ///
    /// Formula: `((i*j) % 3 + (i*j)) % 2 = 0`
    Mask6 = 0b110,
    #[doc = include_str!("../../../assets/masks/mask111.svg")]
    ///
    /// Mask based on position and binary sum of the module's row and column
    /// indices.
    ///
    /// Formula: `((i*j) % 3 + i + j) % 2 = 0`
    Mask7 = 0b111,
}

/// Version (format) of Pharmazentralnummer (PZN) to encode.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PZNVersion {
    /// PZN8 (current standard since 2013).
    ///
    /// Encodes up to 7 digits, zero-padded on the left. An 8-digit input is
    /// accepted in which case Zint validates the supplied check digit.
    #[default]
    PZN8,
    /// PZN7 (obsolete since 2013).
    ///
    /// Encodes up to 7 digits. A modulo-11 check digit is added, or if 7
    /// digits are supplied the check digit is validated.
    PZN7,
}

/// Error correction level for PDF417 and HIBC PDF417.
///
/// The number of codewords used for error correction is determined by
/// `2^(level + 1)`. Higher levels provide more error recovery but reduce
/// data capacity.
///
/// Default level is determined automatically by zint based on the amount of
/// data being encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PDF417ErrorCorrection {
    /// `2^1 = 2` error correction codewords.
    L0 = 0,
    /// `2^2 = 4` error correction codewords.
    L1 = 1,
    /// `2^3 = 8` error correction codewords.
    L2 = 2,
    /// `2^4 = 16` error correction codewords.
    L3 = 3,
    /// `2^5 = 32` error correction codewords.
    L4 = 4,
    /// `2^6 = 64` error correction codewords.
    L5 = 5,
    /// `2^7 = 128` error correction codewords.
    L6 = 6,
    /// `2^8 = 256` error correction codewords.
    L7 = 7,
    /// `2^9 = 512` error correction codewords.
    L8 = 8,
}

/// Number of data columns for MicroPDF417.
///
/// MicroPDF417 supports 1 to 4 data columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MicroPDF417Columns {
    /// 1 data column.
    One = 1,
    /// 2 data columns.
    Two = 2,
    /// 3 data columns.
    Three = 3,
    /// 4 data columns.
    Four = 4,
}

/// Error correction level for Aztec Code and HIBC Aztec Code.
///
/// Specifies the minimum percentage of symbol area used for error correction.
/// If both [`AztecErrorCorrection`] and a symbol size are specified, the size
/// takes precedence and this option is ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AztecErrorCorrection {
    /// At least 10% + 3 codewords of error correction.
    L1 = 1,
    /// At least 23% + 3 codewords of error correction.
    L2 = 2,
    /// At least 36% + 3 codewords of error correction.
    L3 = 3,
    /// At least 50% + 3 codewords of error correction.
    L4 = 4,
}

/// Symbol size for Aztec Code and HIBC Aztec Code.
///
/// Values 1-4 are compact (small bullseye) symbols; values 5-36 are
/// full-range symbols. If a size is specified, the error correction level
/// option is ignored.
///
/// See the zint manual for a full table of symbol sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AztecSize(u8);

impl AztecSize {
    /// Creates an Aztec size from a version number in 1-36.
    ///
    /// Returns `None` if `version` is 0 or greater than 36.
    pub fn from_version(version: u8) -> Option<Self> {
        if version >= 1 && version <= 36 {
            Some(Self(version))
        } else {
            None
        }
    }

    /// Returns the raw version number (1-36).
    pub fn version(self) -> u8 {
        self.0
    }

    /// Returns `true` if this is a compact (small bullseye) symbol
    /// (versions 1-4).
    pub fn is_compact(self) -> bool {
        self.0 <= 4
    }
}

/// Shape mode for Data Matrix and HIBC Data Matrix automatic size selection.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DataMatrixShape {
    /// Allow any shape (default). Zint selects the smallest symbol.
    #[default]
    Any,
    /// Force square symbols only (versions 1-24, sizes 10×10 to 144×144).
    ///
    /// Corresponds to zint `option_3 = DM_SQUARE`.
    Square,
    /// Allow Data Matrix Rectangular Extension (DMRE) symbols in addition to
    /// standard rectangular symbols. Has no effect when a specific size is
    /// chosen via `option_2`.
    ///
    /// Corresponds to zint `option_3 = DM_DMRE`.
    AllowDMRE,
}
