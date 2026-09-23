//! Domain shared value objects: Order, Position, Trade, Portfolio, and shared enums.
//!
//! This crate provides the L0 shared type layer used across all domain crates.

#![cfg_attr(
    test,
    allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)
)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]

/// Re-export `decimalx::Decimal`（ADR-007 唯一底座；kp4 自 rust_decimal 收敛）。
pub use decimalx::{Decimal, DecimalError};

// ---------------------------------------------------------------------------
// InstrumentKey（core 平面唯一 canonical owner；ADR-001 / DX-CAN-001）
// ---------------------------------------------------------------------------

/// 跨层共享的结构化 instrument 标识（`exchange` + `symbol`）。
///
/// 本类型是交易对象共享的 instrument 身份表示，避免在领域类型中重复定义该结构。
/// 它不依赖外部归一化或 I/O crate；wire 层的原生场所符号仍可使用字符串表示。
///
/// 形状边界：只承载 `(exchange, symbol)` 二元组，**不**做跨场所符号归一化、不做
/// 产品线推断、不做校验。exchange 应为稳定小写 provider key（如 `binance`），
/// symbol 为 provider 规范化后的原生标的。adapter 负责保存双向映射。
///
/// serde wire：`{"exchange": "..", "symbol": ".."}`（camelCase 单词无差异）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentKey {
    /// Exchange / provider key（小写，如 `binance`、`okx`、`coinglass`）。
    pub exchange: String,
    /// Provider 原生交易对符号（如 `BTCUSDT`、`BTC-USDT`）。
    pub symbol: String,
}

mod validate;
pub use validate::{
    ValidationError, validate_created_before_updated, validate_gtd_deadline,
    validate_non_negative_quantities, validate_order, validate_order_prices,
    validate_quantity_balance,
};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

/// Unique order identifier (exchange-assigned).
pub type OrderId = String;
/// Unique trade identifier (exchange-assigned).
pub type TradeId = String;
/// Unique execution report identifier.
pub type ReportId = String;
/// Unique position identifier.
pub type PositionId = String;
/// Unique portfolio identifier.
pub type PortfolioId = String;
/// Unix timestamp in milliseconds since 1970-01-01 UTC.
pub type Timestamp = i64;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Side of an order or trade.
///
/// # Examples
///
/// ```
/// let side = domainx::OrderSide::Buy;
/// assert_eq!(side, domainx::OrderSide::Buy);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderSide {
    /// 买入方向。
    Buy,
    /// 卖出方向。
    Sell,
}

/// Type of an order.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderType {
    /// 按市场可用价格立即成交。
    Market,
    /// 价格达到指定限价时成交。
    Limit,
    /// 触发后按市场价格成交。
    StopMarket,
    /// 触发后按指定限价成交。
    StopLimit,
}

/// Status of an order.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderStatus {
    /// 已创建，尚未成交。
    New,
    /// 已部分成交，仍有剩余数量。
    PartiallyFilled,
    /// 全部数量已成交。
    Filled,
    /// 已取消。
    Canceled,
    /// 已拒绝。
    Rejected,
    /// 已过期。
    Expired,
    /// 创建请求处理中。
    PendingNew,
    /// 取消请求处理中。
    PendingCancel,
    /// 修改请求处理中。
    PendingReplace,
}

/// Time-in-force policy for an order.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum TimeInForce {
    /// Good Till Cancelled.
    Gtc,
    /// Immediate Or Cancel.
    Ioc,
    /// Fill Or Kill.
    Fok,
    /// Good Till Date：截止时间戳为 UTC Unix 毫秒，不得早于订单创建时间。
    Gtd(Timestamp),
}

/// Direction of a position.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PositionDirection {
    /// 多头方向。
    Long,
    /// 空头方向。
    Short,
    /// 当前无方向或无持仓。
    Flat,
}

/// Status of a position.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PositionStatus {
    /// 持仓处于开放状态。
    Open,
    /// 持仓已关闭。
    Closed,
    /// 持仓已被强制平仓。
    Liquidated,
}

/// Execution type for an execution report.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExecType {
    /// 新订单已确认。
    New,
    /// 订单已取消。
    Canceled,
    /// 订单已修改。
    Replaced,
    /// 订单已拒绝。
    Rejected,
    /// 发生了成交。
    Trade,
    /// 订单已过期。
    Expired,
    /// 已撤销先前报告的成交。
    TradeCancel,
    /// 状态报告，不改变订单状态。
    Status,
}

// ---------------------------------------------------------------------------
// Commission
// --------------------------------------------------------------------------

/// Commission details for a trade.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Commission {
    /// Commission amount.
    pub amount: Decimal,
    /// Asset in which the commission is denominated.
    pub asset: String,
}

// ---------------------------------------------------------------------------
// Order
// ---------------------------------------------------------------------------

/// A trading order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Exchange-assigned order identifier.
    pub order_id: OrderId,
    /// Instrument 标识（canonical `InstrumentKey`；ADR-001 / DX-CAN-001）。
    pub instrument: InstrumentKey,
    /// Order side (buy / sell).
    pub side: OrderSide,
    /// Order type (market, limit, stop, etc.).
    pub order_type: OrderType,
    /// Current order status.
    pub status: OrderStatus,
    /// Limit price (optional, `None` for market orders).
    pub price: Option<Decimal>,
    /// Stop / trigger price for StopMarket / StopLimit orders.
    pub stop_price: Option<Decimal>,
    /// Ordered quantity.
    pub quantity: Decimal,
    /// Quantity that has been filled.
    pub filled_quantity: Decimal,
    /// Quantity remaining to fill.
    pub remaining_quantity: Decimal,
    /// Average fill price (available after partial / full fill).
    pub avg_fill_price: Option<Decimal>,
    /// Time-in-force policy.
    pub time_in_force: TimeInForce,
    /// Order creation timestamp (Unix ms).
    pub created_at: Timestamp,
    /// Last update timestamp (Unix ms).
    pub updated_at: Timestamp,
    /// Client-supplied order identifier (optional).
    pub client_order_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Position
// ---------------------------------------------------------------------------

/// A trading position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    /// Position identifier.
    pub position_id: PositionId,
    /// Instrument 标识（canonical `InstrumentKey`；ADR-001 / DX-CAN-001）。
    pub instrument: InstrumentKey,
    /// Position direction (long / short / flat).
    pub direction: PositionDirection,
    /// 持仓状态（可选；历史 fixture 可缺省，DX-COMP-001）。
    #[serde(default)]
    pub status: Option<PositionStatus>,
    /// Position quantity.
    pub quantity: Decimal,
    /// Average entry price.
    pub entry_price: Decimal,
    /// Current market price.
    pub current_price: Decimal,
    /// Unrealised P&L.
    pub unrealized_pnl: Decimal,
    /// Realised P&L.
    pub realized_pnl: Decimal,
    /// Position creation timestamp (Unix ms).
    pub created_at: Timestamp,
    /// Last update timestamp (Unix ms).
    pub updated_at: Timestamp,
}

// ---------------------------------------------------------------------------
// Trade
// ---------------------------------------------------------------------------

/// A matched trade (fill).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// Trade identifier.
    pub trade_id: TradeId,
    /// Parent order identifier.
    pub order_id: OrderId,
    /// Instrument 标识（canonical `InstrumentKey`；ADR-001 / DX-CAN-001）。
    pub instrument: InstrumentKey,
    /// Trade side.
    pub side: OrderSide,
    /// Execution price.
    pub price: Decimal,
    /// Filled quantity.
    pub quantity: Decimal,
    /// Commission charged (optional).
    pub commission: Option<Commission>,
    /// Execution timestamp (Unix ms).
    pub executed_at: Timestamp,
    /// Whether the trade was a maker (`true`), taker (`false`), or unknown (`None`).
    pub is_maker: Option<bool>,
}

// ---------------------------------------------------------------------------
// ExecutionReport
// ---------------------------------------------------------------------------

/// An execution report sent by the exchange in response to order lifecycle events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionReport {
    /// Report identifier.
    pub report_id: ReportId,
    /// Related order identifier.
    pub order_id: OrderId,
    /// Execution type describing the event.
    pub exec_type: ExecType,
    /// New order status after this event.
    pub order_status: OrderStatus,
    /// Instrument 标识（canonical `InstrumentKey`；ADR-001 / DX-CAN-001）。
    pub instrument: InstrumentKey,
    /// Order side.
    pub side: OrderSide,
    /// Order type.
    pub order_type: OrderType,
    /// Limit price (optional).
    pub price: Option<Decimal>,
    /// Order quantity.
    pub quantity: Decimal,
    /// Price of the last fill (optional).
    pub last_filled_price: Option<Decimal>,
    /// Quantity of the last fill (optional).
    pub last_filled_quantity: Option<Decimal>,
    /// Cumulative filled quantity.
    pub cumulative_filled_quantity: Decimal,
    /// Remaining quantity.
    pub remaining_quantity: Decimal,
    /// Commission charged (optional).
    pub commission: Option<Commission>,
    /// Trade identifier of the last fill (optional).
    pub trade_id: Option<TradeId>,
    /// Reason for rejection (optional).
    pub reject_reason: Option<String>,
    /// Timestamp of this event (Unix ms).
    pub occurred_at: Timestamp,
}

// ---------------------------------------------------------------------------
// Portfolio
// ---------------------------------------------------------------------------

/// A portfolio (collection of positions with aggregate P&L).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Portfolio {
    /// Portfolio identifier.
    pub portfolio_id: PortfolioId,
    /// Account identifier that owns this portfolio.
    pub account_id: String,
    /// Positions held in the portfolio.
    pub positions: Vec<Position>,
    /// Total unrealised P&L across all positions.
    pub total_unrealized_pnl: Decimal,
    /// Total realised P&L across all positions.
    pub total_realized_pnl: Decimal,
    /// Total commission accrued.
    ///
    /// 单资产汇总或调用方约定的主资产合计；多资产明细见 `commissions`（DX-COMP-001）。
    pub total_commission: Decimal,
    /// 按资产拆分的手续费明细（可空；与 `total_commission` 并存）。
    #[serde(default)]
    pub commissions: Vec<Commission>,
    /// Total number of trades executed.
    pub total_trades: u64,
    /// Last update timestamp (Unix ms).
    pub updated_at: Timestamp,
}

// ---------------------------------------------------------------------------
// DX-COMP-001 helpers
// ---------------------------------------------------------------------------

/// 将多资产手续费按 asset 汇总（同资产 amount 相加）。
///
/// 使用 [`Decimal::checked_add`]；溢出时返回 [`ValidationError::Quantity`]。
pub fn aggregate_commissions_by_asset(
    items: &[Commission],
) -> Result<Vec<Commission>, ValidationError> {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<&str, Decimal> = BTreeMap::new();
    for c in items {
        let entry = map.entry(c.asset.as_str()).or_insert(Decimal::ZERO);
        let sum = entry.checked_add(c.amount).map_err(|e| {
            ValidationError::Quantity(
                format!(
                    "佣金按资产 {} 汇总溢出: current({entry}) + amount({})：{e}",
                    c.asset, c.amount
                ),
                Some(e),
            )
        })?;
        *entry = sum;
    }
    Ok(map
        .into_iter()
        .map(|(asset, amount)| Commission { amount, asset: asset.to_string() })
        .collect())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn ik(sym: &str) -> InstrumentKey {
        InstrumentKey { exchange: "binance".into(), symbol: sym.into() }
    }

    #[test]
    fn test_order_creation() {
        let order = Order {
            order_id: "abc123".into(),
            instrument: ik("BTCUSDT"),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            status: OrderStatus::New,
            price: Some(Decimal::new(50000, 0)),
            stop_price: None,
            quantity: Decimal::new(1, 0),
            filled_quantity: Decimal::ZERO,
            remaining_quantity: Decimal::new(1, 0),
            avg_fill_price: None,
            time_in_force: TimeInForce::Gtc,
            created_at: 1_000_000_000_000,
            updated_at: 1_000_000_000_000,
            client_order_id: Some("my-order".into()),
        };
        assert_eq!(order.order_id, "abc123");
        assert_eq!(order.side, OrderSide::Buy);
        assert_eq!(order.status, OrderStatus::New);
    }

    #[test]
    fn test_order_serialization() {
        let order = Order {
            order_id: "abc".into(),
            instrument: ik("BTCUSDT"),
            side: OrderSide::Sell,
            order_type: OrderType::Market,
            status: OrderStatus::Filled,
            price: None,
            stop_price: None,
            quantity: Decimal::new(2, 0),
            filled_quantity: Decimal::new(2, 0),
            remaining_quantity: Decimal::ZERO,
            avg_fill_price: Some(Decimal::new(49000, 0)),
            time_in_force: TimeInForce::Ioc,
            created_at: 1_000_000_000_000,
            updated_at: 1_000_000_001_000,
            client_order_id: None,
        };
        let json = serde_json::to_string(&order).expect("serialize order");
        let deserialized: Order = serde_json::from_str(&json).expect("deserialize order");
        assert_eq!(order, deserialized);
    }

    #[test]
    fn test_position_lifecycle() {
        let pos = Position {
            position_id: "pos1".into(),
            instrument: ik("ETHUSDT"),
            direction: PositionDirection::Long,
            status: None,
            quantity: Decimal::new(10, 0),
            entry_price: Decimal::new(3000, 0),
            current_price: Decimal::new(3100, 0),
            unrealized_pnl: Decimal::new(1000, 0),
            realized_pnl: Decimal::ZERO,
            created_at: 1_000_000_000_000,
            updated_at: 1_000_000_001_000,
        };
        assert_eq!(pos.direction, PositionDirection::Long);
        assert!(pos.unrealized_pnl > Decimal::ZERO);
    }

    #[test]
    fn test_trade_with_commission() {
        let trade = Trade {
            trade_id: "t1".into(),
            order_id: "o1".into(),
            instrument: ik("BTCUSDT"),
            side: OrderSide::Buy,
            price: Decimal::new(50000, 0),
            quantity: Decimal::new(1, 0),
            commission: Some(Commission { amount: Decimal::new(10, 0), asset: "BTC".into() }),
            executed_at: 1_000_000_000_000,
            is_maker: Some(true),
        };
        let json = serde_json::to_string(&trade).expect("serialize trade");
        let deserialized: Trade = serde_json::from_str(&json).expect("deserialize trade");
        assert_eq!(trade, deserialized);
    }

    #[test]
    fn test_time_in_force_gtd() {
        let tif = TimeInForce::Gtd(1_700_000_000_000);
        let json = serde_json::to_string(&tif).expect("serialize tif");
        let deserialized: TimeInForce = serde_json::from_str(&json).expect("deserialize tif");
        assert_eq!(tif, deserialized);
    }

    #[test]
    fn test_execution_report() {
        let report = ExecutionReport {
            report_id: "r1".into(),
            order_id: "o1".into(),
            exec_type: ExecType::Trade,
            order_status: OrderStatus::PartiallyFilled,
            instrument: ik("BTCUSDT"),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Some(Decimal::new(50000, 0)),
            quantity: Decimal::new(1, 0),
            last_filled_price: Some(Decimal::new(50000, 0)),
            last_filled_quantity: Some(Decimal::new(5, 1)),
            cumulative_filled_quantity: Decimal::new(5, 1),
            remaining_quantity: Decimal::new(5, 1),
            commission: Some(Commission { amount: Decimal::new(1, 2), asset: "BNB".into() }),
            trade_id: Some("t1".into()),
            reject_reason: None,
            occurred_at: 1_000_000_000_000,
        };
        let json = serde_json::to_string(&report).expect("serialize report");
        let deserialized: ExecutionReport =
            serde_json::from_str(&json).expect("deserialize report");
        assert_eq!(report, deserialized);
    }

    #[test]
    fn test_portfolio() {
        let portfolio = Portfolio {
            portfolio_id: "pf1".into(),
            account_id: "acc1".into(),
            positions: vec![Position {
                position_id: "pos1".into(),
                instrument: ik("BTCUSDT"),
                direction: PositionDirection::Long,
                status: None,
                quantity: Decimal::new(1, 0),
                entry_price: Decimal::new(50000, 0),
                current_price: Decimal::new(51000, 0),
                unrealized_pnl: Decimal::new(1000, 0),
                realized_pnl: Decimal::ZERO,
                created_at: 1_000_000_000_000,
                updated_at: 1_000_000_001_000,
            }],
            total_unrealized_pnl: Decimal::new(1000, 0),
            total_realized_pnl: Decimal::ZERO,
            total_commission: Decimal::new(50, 0),
            commissions: vec![Commission { amount: Decimal::new(50, 0), asset: "USDT".into() }],
            total_trades: 5,
            updated_at: 1_000_000_001_000,
        };
        assert_eq!(portfolio.positions.len(), 1);
    }
}
