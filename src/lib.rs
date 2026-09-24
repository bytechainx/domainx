//! 交易领域共享值对象：订单、持仓、成交、组合及共享枚举。
//!
//! 本 crate 为所有领域 crate 提供 L0 共享类型层。

#![cfg_attr(
    test,
    allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)
)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]

/// 公开重导出 `decimalx::Decimal`（ADR-007 唯一底座；kp4 自 `rust_decimal` 收敛）。
pub use decimalx::{Decimal, DecimalError};

// ---------------------------------------------------------------------------
// InstrumentKey（core 平面唯一 canonical owner；ADR-001 / DX-CAN-001）
// ---------------------------------------------------------------------------

/// 跨层共享的结构化标的标识（`exchange` + `symbol`）。
///
/// 本类型是交易对象共享的标的身份表示，避免在领域类型中重复定义该结构。
/// 它不依赖外部归一化或 I/O crate；wire 层的原生场所符号仍可使用字符串表示。
///
/// 形状边界：只承载 `(exchange, symbol)` 二元组，**不**做跨场所符号归一化、不做
/// 产品线推断、不做校验。`exchange` 应为稳定小写 provider key（如 `binance`），
/// `symbol` 为 provider 规范化后的原生标的。适配器负责保存双向映射。
///
/// serde wire：`{"exchange": "..", "symbol": ".."}`（camelCase 单词不改变字段名）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentKey {
    /// 交易场所或 provider 标识（小写，如 `binance`、`okx`、`coinglass`）。
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

/// 唯一订单标识，由交易场所分配。
pub type OrderId = String;
/// 唯一成交标识，由交易场所分配。
pub type TradeId = String;
/// 唯一执行回报标识。
pub type ReportId = String;
/// 唯一持仓标识。
pub type PositionId = String;
/// 唯一组合标识。
pub type PortfolioId = String;
/// 自 1970-01-01 UTC 起计算的 Unix 毫秒时间戳。
pub type Timestamp = i64;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// 订单或成交方向。
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

/// 订单类型。
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

/// 订单状态。
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

/// 订单有效期策略。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum TimeInForce {
    /// 持续有效，直到取消。
    Gtc,
    /// 立即成交，否则取消剩余部分。
    Ioc,
    /// 全部立即成交，否则全部取消。
    Fok,
    /// 指定截止日期：截止时间为 UTC Unix 毫秒，且不得早于订单创建时间。
    Gtd(Timestamp),
}

/// 持仓方向。
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

/// 持仓状态。
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

/// 执行回报中的执行类型。
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

/// 成交手续费明细。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Commission {
    /// 手续费金额。
    pub amount: Decimal,
    /// 手续费计价资产。
    pub asset: String,
}

// ---------------------------------------------------------------------------
// Order
// ---------------------------------------------------------------------------

/// 交易订单。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// 交易场所分配的订单标识。
    pub order_id: OrderId,
    /// 标的标识（规范类型为 `InstrumentKey`；ADR-001 / DX-CAN-001）。
    pub instrument: InstrumentKey,
    /// 订单方向（买入或卖出）。
    pub side: OrderSide,
    /// 订单类型（市价、限价、止损等）。
    pub order_type: OrderType,
    /// 当前订单状态。
    pub status: OrderStatus,
    /// 限价；市价单为 `None`。
    pub price: Option<Decimal>,
    /// StopMarket / StopLimit 订单的止损或触发价格。
    pub stop_price: Option<Decimal>,
    /// 下单数量。
    pub quantity: Decimal,
    /// 已成交数量。
    pub filled_quantity: Decimal,
    /// 剩余待成交数量。
    pub remaining_quantity: Decimal,
    /// 平均成交价格，部分或全部成交后可用。
    pub avg_fill_price: Option<Decimal>,
    /// 订单有效期策略。
    pub time_in_force: TimeInForce,
    /// 订单创建时间，Unix 毫秒。
    pub created_at: Timestamp,
    /// 最近更新时间，Unix 毫秒。
    pub updated_at: Timestamp,
    /// 客户端提供的订单标识，可选。
    pub client_order_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Position
// ---------------------------------------------------------------------------

/// 交易持仓。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    /// 持仓标识。
    pub position_id: PositionId,
    /// 标的标识（规范类型为 `InstrumentKey`；ADR-001 / DX-CAN-001）。
    pub instrument: InstrumentKey,
    /// 持仓方向（多头、空头或空仓）。
    pub direction: PositionDirection,
    /// 持仓状态（可选；历史 fixture 可缺省，DX-COMP-001）。
    #[serde(default)]
    pub status: Option<PositionStatus>,
    /// 持仓数量。
    pub quantity: Decimal,
    /// 平均开仓价格。
    pub entry_price: Decimal,
    /// 当前市场价格。
    pub current_price: Decimal,
    /// 未实现盈亏。
    pub unrealized_pnl: Decimal,
    /// 已实现盈亏。
    pub realized_pnl: Decimal,
    /// 持仓创建时间，Unix 毫秒。
    pub created_at: Timestamp,
    /// 最近更新时间，Unix 毫秒。
    pub updated_at: Timestamp,
}

// ---------------------------------------------------------------------------
// Trade
// ---------------------------------------------------------------------------

/// 已撮合的成交。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// 成交标识。
    pub trade_id: TradeId,
    /// 父订单标识。
    pub order_id: OrderId,
    /// 标的标识（规范类型为 `InstrumentKey`；ADR-001 / DX-CAN-001）。
    pub instrument: InstrumentKey,
    /// 成交方向。
    pub side: OrderSide,
    /// 成交价格。
    pub price: Decimal,
    /// 成交数量。
    pub quantity: Decimal,
    /// 手续费，可选。
    pub commission: Option<Commission>,
    /// 成交时间，Unix 毫秒。
    pub executed_at: Timestamp,
    /// 是否为 maker 成交（`true`）、taker 成交（`false`），或未知（`None`）。
    pub is_maker: Option<bool>,
}

// ---------------------------------------------------------------------------
// ExecutionReport
// ---------------------------------------------------------------------------

/// 交易场所针对订单生命周期事件发送的执行回报。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionReport {
    /// 回报标识。
    pub report_id: ReportId,
    /// 关联订单标识。
    pub order_id: OrderId,
    /// 描述该事件的执行类型。
    pub exec_type: ExecType,
    /// 事件发生后的订单状态。
    pub order_status: OrderStatus,
    /// 标的标识（规范类型为 `InstrumentKey`；ADR-001 / DX-CAN-001）。
    pub instrument: InstrumentKey,
    /// 订单方向。
    pub side: OrderSide,
    /// 订单类型。
    pub order_type: OrderType,
    /// 限价，可选。
    pub price: Option<Decimal>,
    /// 订单数量。
    pub quantity: Decimal,
    /// 最近一笔成交价格，可选。
    pub last_filled_price: Option<Decimal>,
    /// 最近一笔成交数量，可选。
    pub last_filled_quantity: Option<Decimal>,
    /// 累计成交数量。
    pub cumulative_filled_quantity: Decimal,
    /// 剩余数量。
    pub remaining_quantity: Decimal,
    /// 手续费，可选。
    pub commission: Option<Commission>,
    /// 最近一笔成交的标识，可选。
    pub trade_id: Option<TradeId>,
    /// 拒绝原因，可选。
    pub reject_reason: Option<String>,
    /// 事件时间，Unix 毫秒。
    pub occurred_at: Timestamp,
}

// ---------------------------------------------------------------------------
// Portfolio
// ---------------------------------------------------------------------------

/// 投资组合，包含持仓及汇总盈亏。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Portfolio {
    /// 组合标识。
    pub portfolio_id: PortfolioId,
    /// 所属账户标识。
    pub account_id: String,
    /// 组合中的持仓。
    pub positions: Vec<Position>,
    /// 所有持仓的未实现盈亏合计。
    pub total_unrealized_pnl: Decimal,
    /// 所有持仓的已实现盈亏合计。
    pub total_realized_pnl: Decimal,
    /// 累计手续费总额。
    ///
    /// 单资产汇总或调用方约定的主资产合计；多资产明细见 `commissions`（DX-COMP-001）。
    pub total_commission: Decimal,
    /// 按资产拆分的手续费明细（可空；与 `total_commission` 并存）。
    #[serde(default)]
    pub commissions: Vec<Commission>,
    /// 已执行的成交笔数。
    pub total_trades: u64,
    /// 最近更新时间，Unix 毫秒。
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
