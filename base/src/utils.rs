use rust_decimal::prelude::FromPrimitive;
use rust_decimal_macros::dec;
use toml;
use std::fs;
use std::str::FromStr;
use regex::Regex;
use std::error::Error;
use crate::{Config};
use crate::params::Symbol;
use rust_decimal::Decimal;


pub fn load_config(config_path: String) -> Result<Config, Box<dyn Error>> {
    let config_content = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&config_content)?;
    Ok(config)
}

pub fn symbol_to_enum(symbol: &str) -> Symbol {
    let re = Regex::new(r"[_/\-]").unwrap();
    let clean_symbol = re.replace_all(&symbol, "").to_uppercase();
    match clean_symbol.as_str() {
        "USDTTWD" => Symbol::USDT_TWD,
        "ETHBTC" => Symbol::ETH_BTC,
        "ARBTWD" => Symbol::ARB_TWD,
        "ARBUSDT" => Symbol::ARB_USDT,
        "BTCTWD" => Symbol::BTC_TWD,
        "BTCUSDT" => Symbol::BTC_USDT,
        "ETHTWD" => Symbol::ETH_TWD,
        "ETHUSDT" => Symbol::ETH_USDT,
        "BNBTWD" => Symbol::BNB_TWD,
        "BNBUSDT" => Symbol::BNB_USDT,
        "MAXTWD" => Symbol::MAX_TWD,
        "MAXUSDT" => Symbol::MAX_USDT,
        "BCHTWD" => Symbol::BCH_TWD,
        "BCHUSDT" => Symbol::BCH_USDT,
        "XRPTWD" => Symbol::XRP_TWD,
        "XRPUSDT" => Symbol::XRP_USDT,
        "BCNTTWD" => Symbol::BCNT_TWD,
        "BCNTUSDT" => Symbol::BCNT_USDT,
        "LINKTWD" => Symbol::LINK_TWD,
        "LINKUSDT" => Symbol::LINK_USDT,
        "SHIBTWD" => Symbol::SHIB_TWD,
        "SHIBUSDT" => Symbol::SHIB_USDT,
        "LTCTWD" => Symbol::LTC_TWD,
        "LTCUSDT" => Symbol::LTC_USDT,
        "APETWD" => Symbol::APE_TWD,
        "APEUSDT" => Symbol::APE_USDT,
        "DOGETWD" => Symbol::DOGE_TWD,
        "DOGEUSDT" => Symbol::DOGE_USDT,
        "DOTTWD" => Symbol::DOT_TWD,
        "DOTUSDT" => Symbol::DOT_USDT,
        "SOLTWD" => Symbol::SOL_TWD,
        "SOLUSDT" => Symbol::SOL_USDT,
        "SANDTWD" => Symbol::SAND_TWD,
        "SANDUSDT" => Symbol::SAND_USDT,
        "USDCTWD" => Symbol::USDC_TWD,
        "USDCUSDT" => Symbol::USDC_USDT,
        "COMPTWD" => Symbol::COMP_TWD,
        "COMPUSDT" => Symbol::COMP_USDT,
        "ADATWD" => Symbol::ADA_TWD,
        "ADAUSDT" => Symbol::ADA_USDT,
        "MATICTWD" => Symbol::MATIC_TWD,
        "MATICUSDT" => Symbol::MATIC_USDT,
        "LOOTTWD" => Symbol::LOOT_TWD,
        "LOOTUSDT" => Symbol::LOOT_USDT,
        "RLYTWD" => Symbol::RLY_TWD,
        "RLYUSDT" => Symbol::RLY_USDT,
        "GRTTWD" => Symbol::GRT_TWD,
        "YFIUSDT" => Symbol::YFI_USDT,
        "ETCTWD" => Symbol::ETC_TWD,
        "ETCUSDT" => Symbol::ETC_USDT,
        "GALATWD" => Symbol::GALA_TWD,
        "MANATWD" => Symbol::MANA_TWD,
        "ALICETWD" => Symbol::ALICE_TWD,
        "LOOKSTWD" => Symbol::LOOKS_TWD,
        "MASKUSDT" => Symbol::MASK_USDT,
        "XTZTWD" => Symbol::XTZ_TWD,
        "GMTTWD" => Symbol::GMT_TWD,
        "GSTTWD" => Symbol::GST_TWD,
        "ENSTWD" => Symbol::ENS_TWD,
        _ => Symbol::UNKNOWN_SYMBOL,
    }
}

pub fn convert_str_to_decimal(numerical: &str) -> Decimal {
    match Decimal::from_str(numerical) {
        Ok(numerical_dec) => numerical_dec,
        Err(err) => dec!(0.0)
    }
}

pub fn convert_f64_to_decimal(numerical: f64) -> Decimal {
    Decimal::from_f64(numerical).unwrap().round_dp(15)
    // Decimal::fr(numerical).unwrap()
}