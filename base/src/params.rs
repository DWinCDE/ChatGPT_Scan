use std::fmt;
use std::str::FromStr;
use serde::Serialize;

pub enum Currency {
    ADA, ALICE, APE, ARB, BAT, BCH, BCNT, BNB, BTC, CCCX, 
    COMP, DOGE, DOT, ENS, EOS, ETC, FMF, GALA, GMT, GNT, 
    GRT, GST, KNC, LINK, LOOKS, LOOT, LTC, MANA, MASK, MATIC, 
    MAX, MITH, OMG, PAL, RLY, SAND, SEELE, SHIB, SOL, TRX, 
    TWD, TWDT, USDC, USDT, XRP, XTZ, YFI, ZRX
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct SymbolPrecision {
    pub price_precision: u64,
    pub amount_precision: u64,
}

impl SymbolPrecision {
    pub fn default() -> Self {
        SymbolPrecision {
            price_precision: 0,
            amount_precision: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Symbol {
    USDT_TWD,
    ETH_BTC,
    ARB_TWD,
    ARB_USDT,
    BTC_TWD,
    BTC_USDT,
    ETH_TWD,
    ETH_USDT,
    BNB_TWD,
    BNB_USDT,
    MAX_TWD,
    MAX_USDT,
    BCH_TWD,
    BCH_USDT,
    XRP_TWD,
    XRP_USDT,
    BCNT_TWD,
    BCNT_USDT,
    LINK_TWD,
    LINK_USDT,
    SHIB_TWD,
    SHIB_USDT,
    LTC_TWD,
    LTC_USDT,
    APE_TWD,
    APE_USDT,
    DOGE_TWD,
    DOGE_USDT,
    DOT_TWD,
    DOT_USDT,
    SOL_TWD,
    SOL_USDT,
    SAND_TWD,
    SAND_USDT,
    USDC_TWD,
    USDC_USDT,
    COMP_TWD,
    COMP_USDT,
    ADA_TWD,
    ADA_USDT,
    MATIC_TWD,
    MATIC_USDT,
    LOOT_TWD,
    LOOT_USDT,
    RLY_TWD,
    RLY_USDT,
    GRT_TWD,
    YFI_USDT,
    ETC_TWD,
    ETC_USDT,
    GALA_TWD,
    MANA_TWD,
    ALICE_TWD,
    LOOKS_TWD,
    MASK_USDT,
    XTZ_TWD,
    GMT_TWD,
    GST_TWD,
    ENS_TWD,
    UNKNOWN_SYMBOL,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol_str = match *self {
            Symbol::USDT_TWD => "USDT_TWD",
            Symbol::ETH_BTC => "ETH_BTC",
            Symbol::ARB_TWD => "ARB_TWD",
            Symbol::ARB_USDT => "ARB_USDT",
            Symbol::BTC_TWD => "BTC_TWD",
            Symbol::BTC_USDT => "BTC_USDT",
            Symbol::ETH_TWD => "ETH_TWD",
            Symbol::ETH_USDT => "ETH_USDT",
            Symbol::BNB_TWD => "BNB_TWD",
            Symbol::BNB_USDT => "BNB_USDT",
            Symbol::MAX_TWD => "MAX_TWD",
            Symbol::MAX_USDT => "MAX_USDT",
            Symbol::BCH_TWD => "BCH_TWD",
            Symbol::BCH_USDT => "BCH_USDT",
            Symbol::XRP_TWD => "XRP_TWD",
            Symbol::XRP_USDT => "XRP_USDT",
            Symbol::BCNT_TWD => "BCNT_TWD",
            Symbol::BCNT_USDT => "BCNT_USDT",
            Symbol::LINK_TWD => "LINK_TWD",
            Symbol::LINK_USDT => "LINK_USDT",
            Symbol::SHIB_TWD => "SHIB_TWD",
            Symbol::SHIB_USDT => "SHIB_USDT",
            Symbol::LTC_TWD => "LTC_TWD",
            Symbol::LTC_USDT => "LTC_USDT",
            Symbol::APE_TWD => "APE_TWD",
            Symbol::APE_USDT => "APE_USDT",
            Symbol::DOGE_TWD => "DOGE_TWD",
            Symbol::DOGE_USDT => "DOGE_USDT",
            Symbol::DOT_TWD => "DOT_TWD",
            Symbol::DOT_USDT => "DOT_USDT",
            Symbol::SOL_TWD => "SOL_TWD",
            Symbol::SOL_USDT => "SOL_USDT",
            Symbol::SAND_TWD => "SAND_TWD",
            Symbol::SAND_USDT => "SAND_USDT",
            Symbol::USDC_TWD => "USDC_TWD",
            Symbol::USDC_USDT => "USDC_USDT",
            Symbol::COMP_TWD => "COMP_TWD",
            Symbol::COMP_USDT => "COMP_USDT",
            Symbol::ADA_TWD => "ADA_TWD",
            Symbol::ADA_USDT => "ADA_USDT",
            Symbol::MATIC_TWD => "MATIC_TWD",
            Symbol::MATIC_USDT => "MATIC_USDT",
            Symbol::LOOT_TWD => "LOOT_TWD",
            Symbol::LOOT_USDT => "LOOT_USDT",
            Symbol::RLY_TWD => "RLY_TWD",
            Symbol::RLY_USDT => "RLY_USDT",
            Symbol::GRT_TWD => "GRT_TWD",
            Symbol::YFI_USDT => "YFI_USDT",
            Symbol::ETC_TWD => "ETC_TWD",
            Symbol::ETC_USDT => "ETC_USDT",
            Symbol::GALA_TWD => "GALA_TWD",
            Symbol::MANA_TWD => "MANA_TWD",
            Symbol::ALICE_TWD => "ALICE_TWD",
            Symbol::LOOKS_TWD => "LOOKS_TWD",
            Symbol::MASK_USDT => "MASK_USDT",
            Symbol::XTZ_TWD => "XTZ_TWD",
            Symbol::GMT_TWD => "GMT_TWD",
            Symbol::GST_TWD => "GST_TWD",
            Symbol::ENS_TWD => "ENS_TWD",
            Symbol::UNKNOWN_SYMBOL => "UNKNOWN_SYMBOL",
        };
        write!(f, "{}", symbol_str)
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum OrderSide {
    BUY,
    SELL,
    UNKNOWN_ORDER_SIDE,
}

#[derive(Debug, Clone, Serialize)]
pub enum OrderType {
    LIMIT,
    MARKET,
    IOC,
    POST_ONLY,
    UNKNOWN_ORDER_TYPE
}

#[derive(Debug, Clone, Serialize)]
pub enum TimeInForce {
    GTC,
    IOC,
    MAKER_ONLY,
    UNKNOWN_TIMEINFORCE
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub enum OrderStatus {
    NEW,
    CANCEL,
    CANCEL_BY_POST_ONLY,
    FILLED,
    PARTIALLY_FILLED,
    UNKNOWN_STATUS
}

pub trait ExchangeParams {
    fn market(&self, symbol: Symbol) -> String;
    fn orderType(&self, order_type: OrderType) -> String;
    fn orderSide(&self, side: OrderSide) -> String;
    fn orderId(&self, order_id: &str) -> String;
}
// =======================================================================================================================================
pub enum UserData {
    ACCOUNT_BALANCE,
    ACCOUNT_ORDERS,
    ACCOUNT_TRADE,
    UNKNOWN_USER_DATA
}