// src/rtds/symbols.rs

use crate::types::tick::RtdsSymbol;
use crate::types::Asset;

pub fn asset_to_rtds_symbol(asset: Asset) -> RtdsSymbol {
    RtdsSymbol::normalized(asset.rtds_symbol())
}

pub fn rtds_symbol_to_asset(symbol: &RtdsSymbol) -> Option<Asset> {
    Asset::from_rtds_symbol(symbol.as_str())
}

pub fn is_supported_rtds_symbol(symbol: &RtdsSymbol) -> bool {
    rtds_symbol_to_asset(symbol).is_some()
}

pub fn configured_rtds_symbols(assets: &[Asset]) -> Vec<RtdsSymbol> {
    assets
        .iter()
        .map(|asset| asset_to_rtds_symbol(*asset))
        .collect()
}
