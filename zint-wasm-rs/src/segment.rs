use std::{fmt::Display, marker::PhantomData};
use zint_sys::zint_seg;

/// The Extended Channel Interpretation protocol provides additional information
/// about the intended interpretation of the message contained within the
/// barcode symbol and even details about the scan itself.
///
/// ECI is specified by QR Code specification ([ISO/IEC
/// 18004](https://www.iso.org/standard/83389.html)) and the [ECI Assignment
/// Register](https://web.aimglobal.org/external/wcpages/wcecommerce/eComItemDetailsPage.aspx?ItemID=805)
/// is maintained by AIM International.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ECI(u32);

impl Default for ECI {
    fn default() -> Self {
        ECI::ISO_8859_1
    }
}

impl ECI {
    /// Sentinel value to indicate no explicit ECI has been set.
    ///
    /// Applies only to symbologies that don't support ECI.
    pub const NONE: ECI = ECI(0);

    /// IBM Code Page 437 (DOS)
    pub const CP437: ECI = ECI(0);
    /// ISO 8859-1 (Latin-1)
    pub const ISO_8859_1: ECI = ECI(3);
    /// ISO 8859-2 (Latin-2, Central Europe)
    pub const ISO_8859_2: ECI = ECI(4);
    /// ISO 8859-3 (Latin-3, South Europe)
    pub const ISO_8859_3: ECI = ECI(5);
    /// ISO 8859-4 (Latin-4, North Europe)
    pub const ISO_8859_4: ECI = ECI(6);
    /// ISO 8859-5 (Cyrillic)
    pub const ISO_8859_5: ECI = ECI(7);
    /// ISO 8859-6 (Arabic)
    pub const ISO_8859_6: ECI = ECI(8);
    /// ISO 8859-7 (Greek)
    pub const ISO_8859_7: ECI = ECI(9);
    /// ISO 8859-8 (Hebrew)
    pub const ISO_8859_8: ECI = ECI(10);
    /// ISO 8859-9 (Turkish)
    pub const ISO_8859_9: ECI = ECI(11);
    /// ISO 8859-10 (Nordic)
    pub const ISO_8859_10: ECI = ECI(12);
    /// ISO 8859-11 (Thai)
    pub const ISO_8859_11: ECI = ECI(13);
    /// ISO 8859-13 (Baltic)
    pub const ISO_8859_13: ECI = ECI(15);
    /// ISO 8859-14 (Celtic)
    pub const ISO_8859_14: ECI = ECI(16);
    /// ISO 8859-15 (Latin-9)
    pub const ISO_8859_15: ECI = ECI(17);
    /// ISO 8859-16 (Latin-10)
    pub const ISO_8859_16: ECI = ECI(18);
    /// Shift JIS (Japanese)
    pub const SHIFT_JIS_JAPANESE: ECI = ECI(20);
    /// Windows-1250 (Central Europe)
    pub const WINDOWS_1250_CENTRAL_EUROPE: ECI = ECI(21);
    /// Windows-1251 (Cyrillic)
    pub const WINDOWS_1251_CYRILLIC: ECI = ECI(22);
    /// Windows-1252 (Western Europe)
    pub const WINDOWS_1256_WESTERN_EUROPE: ECI = ECI(23);
    /// Windows-1256 (Arabic)
    pub const WINDOWS_1256_ARABIC: ECI = ECI(24);
    /// UTF-16 Big Endian
    pub const UTF_16_BE: ECI = ECI(25);
    /// UTF-8
    pub const UTF_8: ECI = ECI(26);
    /// US-ASCII
    pub const US_ASCII: ECI = ECI(27);
    /// Big5 (Traditional Chinese)
    pub const BIG5: ECI = ECI(28);
    /// GB/T 18030 (Simplified Chinese)
    pub const GB18030: ECI = ECI(29);
    /// KS X 1001 (South Korean)
    ///
    /// Sometimes used for EUC-KR as well
    pub const KS_X_1001: ECI = ECI(30);
    /// GBK (Chinese)
    pub const GBK: ECI = ECI(31);
    /// GB 18030 (Chinese, supersedes GBK)
    pub const GB_18030: ECI = ECI(32);
    /// UTF-16 Little Endian
    pub const UTF_16_LE: ECI = ECI(33);
    /// UTF-32 Big Endian
    pub const UTF_32_BE: ECI = ECI(34);
    /// UTF-32 Little Endian
    pub const UTF_32_LE: ECI = ECI(35);
    /// ISO/IEC 646 invariant (7-bit ASCII subset)
    pub const ISO_646_INV: ECI = ECI(170);
    /// Binary data (no character set interpretation)
    pub const BINARY: ECI = ECI(899);

    // This list is non-exhaustive, you can use new to construct other values.

    /// Construct a new ECI code from `u32` value.
    ///
    /// Returns [`ECIError`] if the value is larger than 999999 or one of: 1, 2, 14, 19.
    pub fn new(value: u32) -> Result<Self, ECIError> {
        if value > 999999 || [1, 2, 14, 19].contains(&value) {
            return Err(ECIError(value));
        }
        Ok(Self(value))
    }
}

/// Barcode segment is a byte slice with associated [`ECI`] value.
///
/// A segment can't be empty.
///
/// Encoded strings don't need to be terminated with a nul byte.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Segment<'d> {
    inner: zint_seg,
    _phantom: PhantomData<&'d [u8]>,
}

impl<'d> Segment<'d> {
    /// This function will panic if `length` is 0 - segments can't be empty.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if any of the following conditions are violated:
    ///
    /// * `data` must point to `length` consecutive properly initialized bytes.
    ///
    /// * The memory referenced by the returned slice must not be mutated for
    ///   the duration of lifetime `'d`.
    ///
    /// * `length` of the slice must be no larger than `i32::MAX`, and adding
    ///   that size to `data` must not "wrap around" the address space.
    pub unsafe fn from_raw_parts(source: *const u8, length: usize, eci: ECI) -> Self {
        if length == 0 {
            panic!("can't construct an empty segment")
        }
        Self {
            inner: zint_seg {
                source: source as *mut u8,
                length: length as i32,
                eci: eci.0 as i32,
            },
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn new(data: &'d [u8], eci: ECI) -> Self {
        // SAFETY: data is valid for 'd and [u8] upholds other safety requirements
        unsafe { Self::from_raw_parts(data.as_ptr(), data.len(), eci) }
    }

    #[inline]
    pub fn new_binary(data: &'d [u8]) -> Self {
        Self::new(data, ECI::BINARY)
    }

    #[inline]
    pub fn new_utf8(string: &'d str) -> Self {
        Self::new(string.as_bytes(), ECI::UTF_8)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.length as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.length != 0
    }

    pub fn eci(&self) -> ECI {
        ECI(self.inner.eci as u32)
    }

    /// Copies segment data into target buffer, with no special ECI handling.
    ///
    /// This function doesn't insert a nul byte at the end of `buffer`
    /// automatically.
    ///
    /// Returned value is the number of copied bytes - always the minimum of
    /// this segment length and target buffer length.
    pub fn write_to_cchar_buffer(&self, buffer: &mut [std::ffi::c_char]) -> usize {
        let copy_count = self.len().min(buffer.len());
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.inner.source as *const i8,
                buffer.as_mut_ptr(),
                copy_count,
            )
        };
        copy_count
    }
}

impl<'d> From<&'d str> for Segment<'d> {
    #[inline]
    fn from(value: &'d str) -> Self {
        Self::new_utf8(value)
    }
}

impl<'d> AsRef<[u8]> for Segment<'d> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.inner.source, self.len()) }
    }
}

impl<'d> std::fmt::Debug for Segment<'d> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Segment")
            .field("data", &self.as_ref())
            .field("eci", &self.eci())
            .finish()
    }
}

/// Returned when ECI value is too large for zint to handle.
#[derive(Debug)]
pub struct ECIError(u32);
impl Display for ECIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            // Wikipedia lists 1 and 2 as alternate values for 0 and 3, but
            // that's not part of the specification and instead likely default
            // handling in some application that was used to source that
            // information. Zint produces invalid argument errors for those
            // values so we handle them the same way.
            1 | 2 | 14 | 19 => write!(f, "ECI {} is unassigned", self.0),
            large => write!(f, "provided ECI value `{large}` is too large"),
        }
    }
}
impl core::error::Error for ECIError {}
