//! Code page and international character set commands.
//!
//! The printer supports 40+ code pages for international character support.

use super::{Command, ESC};

/// Character code page selection.
///
/// The printer supports 40+ code pages for international character support.
/// Use `ESC t n` to select a code page.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodePage {
    /// CP437 - USA, Standard Europe. Default code page.
    #[default]
    Cp437UsaStandardEurope = 0,
    /// Katakana - Japanese syllabary.
    Katakana = 1,
    /// CP850 - Multilingual Latin.
    Cp850Multilingual = 2,
    /// CP860 - Portuguese.
    Cp860Portuguese = 3,
    /// CP863 - Canadian-French.
    Cp863CanadianFrench = 4,
    /// CP865 - Nordic (Danish, Norwegian, Swedish, Finnish).
    Cp865Nordic = 5,
    /// Windows-1252 - Western European Latin I.
    Windows1252LatinI = 16,
    /// CP866 - Cyrillic #2 (Russian).
    Cp866Cyrillic2 = 17,
    /// CP852 - Latin 2 (Central/Eastern European).
    Cp852Latin2 = 18,
    /// CP858 - Multilingual with Euro symbol.
    Cp858Euro = 19,
    /// CP862 - Hebrew (DOS).
    Cp862HebrewDos = 21,
    /// CP864 - Arabic.
    Cp864Arabic = 22,
    /// Thai character code 42.
    Thai42 = 23,
    /// Windows-1253 - Greek.
    Windows1253Greek = 24,
    /// Windows-1254 - Turkish.
    Windows1254Turkish = 25,
    /// Windows-1257 - Baltic (Lithuanian, Latvian, Estonian).
    Windows1257Baltic = 26,
    /// Farsi - Persian.
    Farsi = 27,
    /// Windows-1251 - Cyrillic (Russian, Bulgarian, Serbian).
    Windows1251Cyrillic = 28,
    /// CP737 - Greek (DOS).
    Cp737Greek = 29,
    /// CP775 - Baltic (DOS).
    Cp775Baltic = 30,
    /// Thai character code 14.
    Thai14 = 31,
    /// Hebrew Old code.
    HebrewOld = 32,
    /// Windows-1255 - Hebrew (Windows).
    Windows1255HebrewNew = 33,
    /// Thai character code 11.
    Thai11 = 34,
    /// Thai character code 18.
    Thai18 = 35,
    /// CP855 - Cyrillic (alternative).
    Cp855Cyrillic = 36,
    /// CP857 - Turkish (DOS).
    Cp857Turkish = 37,
    /// CP928 - Greek (alternative).
    Cp928Greek = 38,
    /// Thai character code 16.
    Thai16 = 39,
    /// Windows-1256 - Arabic (Windows).
    Windows1256Arabic = 40,
}

impl CodePage {
    /// Get the numeric value for ESC t command.
    pub const fn as_byte(self) -> u8 {
        self as u8
    }
}

/// Select character code page.
///
/// Changes the character encoding for subsequent text.
///
/// ESC/POS: `ESC t n` (0x1B 0x74 n)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectCodePage(pub CodePage);

impl Command for SelectCodePage {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b't', self.0.as_byte()]
    }
}

/// International character set selection.
///
/// Affects specific characters like currency symbols and punctuation
/// that vary between countries.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InternationalCharacterSet {
    /// U.S.A. - Standard ASCII.
    #[default]
    Usa = 0,
    /// France - French accented characters.
    France = 1,
    /// Germany - German umlauts and eszett.
    Germany = 2,
    /// U.K. - British pound symbol.
    Uk = 3,
    /// Denmark I - Danish/Norwegian characters.
    DenmarkI = 4,
    /// Sweden - Swedish characters.
    Sweden = 5,
    /// Italy - Italian characters.
    Italy = 6,
    /// Spain I - Spanish characters with ñ.
    SpainI = 7,
    /// Japan - Japanese characters.
    Japan = 8,
    /// Norway - Norwegian characters.
    Norway = 9,
    /// Denmark II - Alternate Danish characters.
    DenmarkII = 10,
    /// Spain II - Alternate Spanish characters.
    SpainII = 11,
    /// Latin America - Latin American Spanish.
    LatinAmerica = 12,
    /// Korea - Korean characters.
    Korea = 13,
}

impl InternationalCharacterSet {
    /// Get the numeric value for ESC R command.
    pub const fn as_byte(self) -> u8 {
        self as u8
    }
}

/// Select international character set.
///
/// Changes country-specific characters for subsequent text.
///
/// ESC/POS: `ESC R n` (0x1B 0x52 n)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectCharacterSet(pub InternationalCharacterSet);

impl Command for SelectCharacterSet {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'R', self.0.as_byte()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codepage_cp437_value() {
        assert_eq!(CodePage::Cp437UsaStandardEurope as u8, 0);
    }

    #[test]
    fn codepage_katakana_value() {
        assert_eq!(CodePage::Katakana as u8, 1);
    }

    #[test]
    fn codepage_windows1252_value() {
        assert_eq!(CodePage::Windows1252LatinI as u8, 16);
    }

    #[test]
    fn codepage_windows1251_value() {
        assert_eq!(CodePage::Windows1251Cyrillic as u8, 28);
    }

    #[test]
    fn codepage_windows1256_value() {
        assert_eq!(CodePage::Windows1256Arabic as u8, 40);
    }

    #[test]
    fn select_codepage_cp437() {
        let cmd = SelectCodePage(CodePage::Cp437UsaStandardEurope);
        assert_eq!(cmd.encode(), vec![0x1B, b't', 0]);
    }

    #[test]
    fn select_codepage_windows1252() {
        let cmd = SelectCodePage(CodePage::Windows1252LatinI);
        assert_eq!(cmd.encode(), vec![0x1B, b't', 16]);
    }

    #[test]
    fn select_codepage_cyrillic() {
        let cmd = SelectCodePage(CodePage::Windows1251Cyrillic);
        assert_eq!(cmd.encode(), vec![0x1B, b't', 28]);
    }

    #[test]
    fn international_charset_usa_value() {
        assert_eq!(InternationalCharacterSet::Usa as u8, 0);
    }

    #[test]
    fn international_charset_france_value() {
        assert_eq!(InternationalCharacterSet::France as u8, 1);
    }

    #[test]
    fn international_charset_korea_value() {
        assert_eq!(InternationalCharacterSet::Korea as u8, 13);
    }

    #[test]
    fn select_charset_usa() {
        let cmd = SelectCharacterSet(InternationalCharacterSet::Usa);
        assert_eq!(cmd.encode(), vec![0x1B, b'R', 0]);
    }

    #[test]
    fn select_charset_germany() {
        let cmd = SelectCharacterSet(InternationalCharacterSet::Germany);
        assert_eq!(cmd.encode(), vec![0x1B, b'R', 2]);
    }

    #[test]
    fn select_charset_japan() {
        let cmd = SelectCharacterSet(InternationalCharacterSet::Japan);
        assert_eq!(cmd.encode(), vec![0x1B, b'R', 8]);
    }

    #[test]
    fn default_codepage_is_cp437() {
        assert_eq!(CodePage::default(), CodePage::Cp437UsaStandardEurope);
    }

    #[test]
    fn default_charset_is_usa() {
        assert_eq!(InternationalCharacterSet::default(), InternationalCharacterSet::Usa);
    }
}
