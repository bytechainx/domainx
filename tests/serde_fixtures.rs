#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! DX-API-002：版本化 JSON fixture round-trip（camelCase / Decimal / TimeInForce）。
//!
//! Decimal wire 为 decimalx v1：`{mantissa, scale}`；**写入** `mantissa` 为十进制字符串（读兼容历史 JSON 整数）。

use domainx::{Decimal, ExecutionReport, Order, Portfolio, TimeInForce, Trade, validate_order};
use serde_json::Value;
use std::str::FromStr;

fn decimal_mantissa_i128(v: &Value) -> Option<i128> {
    let m = v.get("mantissa")?;
    if let Some(s) = m.as_str() { s.parse().ok() } else { m.as_i64().map(i128::from) }
}

/// DX-API-002
#[test]
fn dx_api_002_fixture_order_limit_round_trip_and_validate() {
    let raw = include_str!("fixtures/order_limit.json");
    let order: Order = serde_json::from_str(raw).expect("deserialize order fixture");
    assert_eq!(order.order_id, "ord-001");
    assert_eq!(order.side, domainx::OrderSide::Buy);
    assert_eq!(order.order_type, domainx::OrderType::Limit);
    assert_eq!(order.price, Some(Decimal::from_str("50000.00").expect("price")));
    assert_eq!(order.quantity, Decimal::from_str("1.5000").expect("qty"));
    // 尾随零由 Decimal 保留 scale
    assert_eq!(order.quantity.scale(), 4);

    validate_order(&order).expect("fixture order must pass DX-VAL");

    let json = serde_json::to_value(&order).expect("serialize");
    // camelCase 字段名
    assert!(json.get("orderId").is_some());
    assert!(json.get("timeInForce").is_some());
    assert!(json.get("clientOrderId").is_some());
    assert!(json.get("order_id").is_none());
    // decimalx wire v1：结构字段；交换剖面写入 string mantissa
    let qty = json.get("quantity").expect("quantity");
    assert_eq!(decimal_mantissa_i128(qty), Some(15000));
    assert_eq!(qty.get("scale").and_then(|v| v.as_u64()), Some(4));
    assert!(qty.get("mantissa").and_then(|v| v.as_str()).is_some());

    let again: Order = serde_json::from_value(json).expect("round-trip");
    assert_eq!(order, again);
}

/// DX-API-002
#[test]
fn dx_api_002_fixture_time_in_force_gtd_adjacently_tagged() {
    let raw = include_str!("fixtures/time_in_force_gtd.json");
    let tif: TimeInForce = serde_json::from_str(raw).expect("deserialize Gtd");
    assert_eq!(tif, TimeInForce::Gtd(1_700_000_001_000));

    let value = serde_json::to_value(&tif).expect("serialize");
    assert_eq!(value["type"], "Gtd");
    assert_eq!(value["value"], 1_700_000_001_000_i64);

    // 其他 TIF 变体
    for (tif, type_name) in
        [(TimeInForce::Gtc, "Gtc"), (TimeInForce::Ioc, "Ioc"), (TimeInForce::Fok, "Fok")]
    {
        let v = serde_json::to_value(&tif).expect("serialize tif");
        assert_eq!(v["type"], type_name, "tif {type_name}");
        let back: TimeInForce = serde_json::from_value(v).expect("round-trip tif");
        assert_eq!(back, tif);
    }
}

/// DX-API-002
#[test]
fn dx_api_002_fixture_trade_decimal_trailing_zeros_negative_large() {
    let raw = include_str!("fixtures/trade_decimal_edge.json");
    let trade: Trade = serde_json::from_str(raw).expect("deserialize trade");

    // 负数价格（fixture 覆盖 Decimal 精度路径；业务上是否允许由上层校验）
    assert_eq!(trade.price, Decimal::from_str("-0.000001").expect("price"));
    assert_eq!(trade.quantity, Decimal::from_str("12345678901234567890.123456789").expect("qty"));
    let commission = trade.commission.as_ref().expect("commission");
    assert_eq!(commission.amount, Decimal::from_str("0.1000").expect("fee"));
    assert_eq!(commission.amount.scale(), 4);

    let json = serde_json::to_string(&trade).expect("serialize");
    // 不得落到 IEEE-754 浮点 JSON number 导致精度损失；
    // decimalx 交换剖面以 string mantissa 承载大数（读仍兼容历史整数 fixture）。
    assert!(
        json.contains("12345678901234567890123456789"),
        "large decimal mantissa must survive in wire JSON, got {json}"
    );
    assert!(
        json.contains("\"scale\":9") || json.contains("\"scale\": 9"),
        "large decimal scale must be preserved, got {json}"
    );
    let again: Trade = serde_json::from_str(&json).expect("round-trip");
    assert_eq!(trade, again);
}

/// DX-API-002
#[test]
fn dx_api_002_fixture_execution_report_camel_case() {
    let raw = include_str!("fixtures/execution_report.json");
    let report: ExecutionReport = serde_json::from_str(raw).expect("deserialize execution report");
    assert_eq!(report.exec_type, domainx::ExecType::Trade);
    assert_eq!(report.order_status, domainx::OrderStatus::PartiallyFilled);
    assert_eq!(
        report.last_filled_quantity,
        Some(Decimal::from_str("0.50").expect("last fill qty"))
    );

    let value = serde_json::to_value(&report).expect("serialize report");
    assert!(value.get("reportId").is_some());
    assert!(value.get("execType").is_some());
    assert!(value.get("lastFilledPrice").is_some());
    assert!(value.get("cumulativeFilledQuantity").is_some());
    assert!(value.get("occurredAt").is_some());

    let again: ExecutionReport = serde_json::from_value(value).expect("round-trip report");
    assert_eq!(report, again);
}

/// DX-API-002
#[test]
fn dx_api_002_enum_variants_use_camel_case_wire_names() {
    use domainx::{ExecType, OrderSide, OrderStatus, OrderType, PositionDirection};

    let cases: &[(&str, serde_json::Value)] = &[
        ("side-buy", serde_json::to_value(OrderSide::Buy).expect("side")),
        ("type-stopMarket", serde_json::to_value(OrderType::StopMarket).expect("type")),
        (
            "status-partiallyFilled",
            serde_json::to_value(OrderStatus::PartiallyFilled).expect("status"),
        ),
        ("dir-long", serde_json::to_value(PositionDirection::Long).expect("dir")),
        ("exec-tradeCancel", serde_json::to_value(ExecType::TradeCancel).expect("exec")),
    ];
    for (label, value) in cases {
        let s = value.as_str().unwrap_or_else(|| panic!("{label} must be string"));
        assert!(
            s.chars().next().is_some_and(|c| c.is_lowercase()) || s.contains(char::is_uppercase),
            "{label}: unexpected wire form {s}"
        );
        // 具体 camelCase 期望
        match *label {
            "side-buy" => assert_eq!(s, "buy"),
            "type-stopMarket" => assert_eq!(s, "stopMarket"),
            "status-partiallyFilled" => assert_eq!(s, "partiallyFilled"),
            "dir-long" => assert_eq!(s, "long"),
            "exec-tradeCancel" => assert_eq!(s, "tradeCancel"),
            _ => {}
        }
    }
}

/// DX-API-002 / T-CONTRACT：snake_case 字段名不得被当成合法 Order wire。
#[test]
fn dx_api_002_order_wire_rejects_snake_case_order_id() {
    let json = r#"{
      "order_id":"ord-001",
      "instrument":{"exchange":"binance","symbol":"BTCUSDT"},
      "side":"buy",
      "orderType":"limit",
      "status":"new",
      "price":{"mantissa":1,"scale":0},
      "quantity":{"mantissa":1,"scale":0},
      "filledQuantity":{"mantissa":0,"scale":0},
      "remainingQuantity":{"mantissa":1,"scale":0},
      "timeInForce":{"type":"Gtc"},
      "createdAt":1,
      "updatedAt":1
    }"#;
    let err = serde_json::from_str::<Order>(json).expect_err("snake_case order_id 必须失败");
    assert!(err.to_string().contains("orderId") || err.to_string().contains("missing field"));
}

/// DX-API-002：六个外部标记枚举的全部 29 个变体使用 camelCase wire 名。
///
/// 与 `src/lib.rs` 的 `enum_variants_require_wire_table_update` 配套：那里守变体集合，
/// 这里守 wire 名。`TimeInForce` 是相邻标记，另由本文件前两条用例覆盖。
#[test]
fn dx_api_002_all_enum_variants_use_camel_case_wire_names() {
    use domainx::{ExecType, OrderSide, OrderStatus, OrderType, PositionDirection, PositionStatus};

    macro_rules! assert_wire {
        ($ty:ty, [$(($variant:expr, $name:literal)),+ $(,)?]) => {{
            let mut count = 0usize;
            $(
                let json = serde_json::to_value(&$variant).expect("serialize enum variant");
                assert_eq!(
                    json,
                    serde_json::Value::String($name.to_string()),
                    "wire name of {}::{}",
                    stringify!($ty),
                    stringify!($variant)
                );
                let back: $ty =
                    serde_json::from_value(json).expect("deserialize enum variant");
                assert_eq!(
                    back,
                    $variant,
                    "round-trip of {}::{}",
                    stringify!($ty),
                    stringify!($variant)
                );
                count += 1;
            )+
            count
        }};
    }

    let total = assert_wire!(OrderSide, [(OrderSide::Buy, "buy"), (OrderSide::Sell, "sell")])
        + assert_wire!(
            OrderType,
            [
                (OrderType::Market, "market"),
                (OrderType::Limit, "limit"),
                (OrderType::StopMarket, "stopMarket"),
                (OrderType::StopLimit, "stopLimit"),
            ]
        )
        + assert_wire!(
            OrderStatus,
            [
                (OrderStatus::New, "new"),
                (OrderStatus::PartiallyFilled, "partiallyFilled"),
                (OrderStatus::Filled, "filled"),
                (OrderStatus::Canceled, "canceled"),
                (OrderStatus::Rejected, "rejected"),
                (OrderStatus::Expired, "expired"),
                (OrderStatus::PendingNew, "pendingNew"),
                (OrderStatus::PendingCancel, "pendingCancel"),
                (OrderStatus::PendingReplace, "pendingReplace"),
            ]
        )
        + assert_wire!(
            PositionDirection,
            [
                (PositionDirection::Long, "long"),
                (PositionDirection::Short, "short"),
                (PositionDirection::Flat, "flat"),
            ]
        )
        + assert_wire!(
            PositionStatus,
            [
                (PositionStatus::Open, "open"),
                (PositionStatus::Closed, "closed"),
                (PositionStatus::Liquidated, "liquidated"),
            ]
        )
        + assert_wire!(
            ExecType,
            [
                (ExecType::New, "new"),
                (ExecType::Canceled, "canceled"),
                (ExecType::Replaced, "replaced"),
                (ExecType::Rejected, "rejected"),
                (ExecType::Trade, "trade"),
                (ExecType::Expired, "expired"),
                (ExecType::TradeCancel, "tradeCancel"),
                (ExecType::Status, "status"),
            ]
        );

    assert_eq!(total, 29, "六个枚举共 29 个变体，须与 src/lib.rs 的穷尽匹配守卫同步");
}

const PORTFOLIO_WITHOUT_COMMISSIONS: &str = r#"{
  "portfolioId":"pf-1",
  "accountId":"acct-1",
  "positions":[],
  "totalUnrealizedPnl":{"mantissa":0,"scale":0},
  "totalRealizedPnl":{"mantissa":0,"scale":0},
  "totalCommission":{"mantissa":0,"scale":0},
  "totalTrades":0,
  "updatedAt":1700000000000
}"#;

fn portfolio_fixture() -> serde_json::Value {
    serde_json::from_str(PORTFOLIO_WITHOUT_COMMISSIONS).expect("portfolio base json")
}

/// DX-API-002：`commissions` 缺失时按 `#[serde(default)]` 得空向量。
#[test]
fn dx_api_002_portfolio_commissions_absent_defaults_to_empty() {
    let portfolio: Portfolio =
        serde_json::from_value(portfolio_fixture()).expect("portfolio without commissions");
    assert!(portfolio.commissions.is_empty());
}

/// DX-API-002：`commissions: null` 不是缺失，须反序列化失败。
#[test]
fn dx_api_002_portfolio_commissions_null_is_rejected() {
    let mut value = portfolio_fixture();
    value["commissions"] = serde_json::Value::Null;
    let err = serde_json::from_value::<Portfolio>(value).expect_err("null commissions must fail");
    assert_eq!(err.classify(), serde_json::error::Category::Data);
}

/// DX-API-002：显式空数组与缺失同结果。
#[test]
fn dx_api_002_portfolio_commissions_explicit_empty() {
    let mut value = portfolio_fixture();
    value["commissions"] = serde_json::json!([]);
    let portfolio: Portfolio =
        serde_json::from_value(value).expect("portfolio with empty commissions");
    assert!(portfolio.commissions.is_empty());
}
