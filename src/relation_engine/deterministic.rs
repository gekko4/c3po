use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageName {
    LongUpPlusShortDown,
    LongDownPlusShortUp,
}

impl PackageName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LongUpPlusShortDown => "LONG_UP_PLUS_SHORT_DOWN",
            Self::LongDownPlusShortUp => "LONG_DOWN_PLUS_SHORT_UP",
        }
    }
}

pub fn decide_package_name(short_ptb: Decimal, long_ptb: Decimal) -> Option<PackageName> {
    if short_ptb > long_ptb {
        Some(PackageName::LongUpPlusShortDown)
    } else if short_ptb < long_ptb {
        Some(PackageName::LongDownPlusShortUp)
    } else {
        None
    }
}