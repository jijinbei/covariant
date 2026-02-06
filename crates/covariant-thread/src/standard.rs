use std::fmt;
use std::str::FromStr;

/// Generates an enum with `Display`, `FromStr`, and an `ALL` constant.
macro_rules! string_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $( $(#[$vmeta:meta])* $variant:ident => $str:literal ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $name {
            $( $(#[$vmeta])* $variant ),*
        }

        impl $name {
            /// All variants of this enum.
            pub const ALL: &[Self] = &[ $( Self::$variant ),* ];
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let s = match self {
                    $( Self::$variant => $str ),*
                };
                f.write_str(s)
            }
        }

        impl FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $( $str => Ok(Self::$variant), )*
                    _ => Err(format!(
                        concat!("unknown ", stringify!($name), ": {}"), s
                    )),
                }
            }
        }
    };
}

string_enum! {
    /// Thread standard families.
    pub enum ThreadStandard {
        /// ISO 261 / ISO 262 metric threads.
        IsoMetric => "ISO",
        /// ASME B1.1 Unified Thread Standard.
        Uts => "UTS",
        /// BS 84 British Standard Whitworth (future).
        Bsw => "BSW",
    }
}

string_enum! {
    /// Individual thread sizes across all standards.
    pub enum ThreadSize {
        // ISO Metric sizes (15 total)
        M1_6  => "M1.6",
        M2    => "M2",
        M2_5  => "M2.5",
        M3    => "M3",
        M4    => "M4",
        M5    => "M5",
        M6    => "M6",
        M8    => "M8",
        M10   => "M10",
        M12   => "M12",
        M14   => "M14",
        M16   => "M16",
        M20   => "M20",
        M24   => "M24",
        M30   => "M30",
        // UTS sizes (13 total)
        /// #2-56 UNC
        Uts2_56    => "#2-56",
        /// #4-40 UNC
        Uts4_40    => "#4-40",
        /// #6-32 UNC
        Uts6_32    => "#6-32",
        /// #8-32 UNC
        Uts8_32    => "#8-32",
        /// #10-24 UNC
        Uts10_24   => "#10-24",
        /// #10-32 UNF
        Uts10_32   => "#10-32",
        /// 1/4"-20 UNC
        Uts1_4_20  => "1/4\"-20",
        /// 5/16"-18 UNC
        Uts5_16_18 => "5/16\"-18",
        /// 3/8"-16 UNC
        Uts3_8_16  => "3/8\"-16",
        /// 7/16"-14 UNC
        Uts7_16_14 => "7/16\"-14",
        /// 1/2"-13 UNC
        Uts1_2_13  => "1/2\"-13",
        /// 5/8"-11 UNC
        Uts5_8_11  => "5/8\"-11",
        /// 3/4"-10 UNC
        Uts3_4_10  => "3/4\"-10",
    }
}

impl ThreadSize {
    /// Returns which standard this size belongs to.
    pub fn standard(self) -> ThreadStandard {
        match self {
            Self::M1_6 | Self::M2 | Self::M2_5 | Self::M3 | Self::M4 | Self::M5
            | Self::M6 | Self::M8 | Self::M10 | Self::M12 | Self::M14 | Self::M16
            | Self::M20 | Self::M24 | Self::M30 => ThreadStandard::IsoMetric,

            Self::Uts2_56 | Self::Uts4_40 | Self::Uts6_32 | Self::Uts8_32
            | Self::Uts10_24 | Self::Uts10_32 | Self::Uts1_4_20 | Self::Uts5_16_18
            | Self::Uts3_8_16 | Self::Uts7_16_14 | Self::Uts1_2_13 | Self::Uts5_8_11
            | Self::Uts3_4_10 => ThreadStandard::Uts,
        }
    }
}

string_enum! {
    /// Thread type (internal/external).
    pub enum ThreadKind {
        /// Internal thread (tapped hole).
        Internal       => "internal",
        /// External thread (bolt/screw).
        External       => "external",
        /// Clearance hole (no threads, close fit).
        ClearanceClose => "clearance-close",
        /// Clearance hole (no threads, medium/normal fit).
        ClearanceMedium => "clearance-medium",
        /// Clearance hole (no threads, free/loose fit).
        ClearanceFree  => "clearance-free",
        /// Insert hole (helicoil or similar).
        Insert         => "insert",
    }
}

string_enum! {
    /// Clearance hole fit categories (ISO 273).
    pub enum ClearanceFit {
        /// Close fit — minimum clearance.
        Close  => "close",
        /// Medium/normal fit.
        Medium => "medium",
        /// Free/loose fit — maximum clearance.
        Free   => "free",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generic round-trip test: Display → FromStr for every variant in ALL.
    fn assert_display_roundtrip<T>(variants: &[T])
    where
        T: fmt::Display + FromStr<Err = String> + PartialEq + std::fmt::Debug + Copy,
    {
        for v in variants {
            let s = v.to_string();
            let parsed: T = s.parse().unwrap();
            assert_eq!(*v, parsed, "roundtrip failed for {s:?}");
        }
    }

    #[test]
    fn thread_standard_roundtrip() {
        assert_display_roundtrip(ThreadStandard::ALL);
    }

    #[test]
    fn thread_standard_from_str_error() {
        assert!("NOPE".parse::<ThreadStandard>().is_err());
    }

    #[test]
    fn thread_size_roundtrip() {
        assert_display_roundtrip(ThreadSize::ALL);
    }

    #[test]
    fn thread_size_iso_standard() {
        for size in &ThreadSize::ALL[..15] {
            assert_eq!(size.standard(), ThreadStandard::IsoMetric, "{size}");
        }
    }

    #[test]
    fn thread_size_uts_standard() {
        for size in &ThreadSize::ALL[15..] {
            assert_eq!(size.standard(), ThreadStandard::Uts, "{size}");
        }
    }

    #[test]
    fn thread_size_all_count() {
        assert_eq!(ThreadSize::ALL.len(), 28); // 15 ISO + 13 UTS
    }

    #[test]
    fn thread_size_from_str_error() {
        assert!("M99".parse::<ThreadSize>().is_err());
    }

    #[test]
    fn thread_kind_roundtrip() {
        assert_display_roundtrip(ThreadKind::ALL);
    }

    #[test]
    fn thread_kind_from_str_error() {
        assert!("nope".parse::<ThreadKind>().is_err());
    }

    #[test]
    fn clearance_fit_roundtrip() {
        assert_display_roundtrip(ClearanceFit::ALL);
    }

    #[test]
    fn clearance_fit_from_str_error() {
        assert!("tight".parse::<ClearanceFit>().is_err());
    }
}
