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
    /// Secondary Message. Zint doesn't automatically split the data between
    /// the Primary and Secondary Messages.
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
