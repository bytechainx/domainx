#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! DX-COMP-001：Position.status 与多资产手续费 fixture。

use domainx::{
    Commission, Decimal, Portfolio, PositionStatus, ValidationError, aggregate_commissions_by_asset,
};
use std::str::FromStr;

/// DX-COMP-001
#[test]
fn dx_comp_001_portfolio_multi_asset_commissions_round_trip() {
    let raw = include_str!("fixtures/portfolio_multi_commission.json");
    let pf: Portfolio = serde_json::from_str(raw).expect("portfolio");
    assert_eq!(pf.positions.len(), 1);
    assert_eq!(pf.positions[0].status, Some(PositionStatus::Open));
    assert_eq!(pf.commissions.len(), 3);

    let aggregated = aggregate_commissions_by_asset(&pf.commissions).expect("aggregate");
    assert_eq!(aggregated.len(), 2);
    let bnb = aggregated.iter().find(|c| c.asset == "BNB").expect("BNB");
    assert_eq!(bnb.amount, Decimal::from_str("0.0015").expect("bnb amount"));
    let usdt = aggregated.iter().find(|c| c.asset == "USDT").expect("USDT");
    assert_eq!(usdt.amount, Decimal::from_str("1.25").expect("usdt amount"));

    let value = serde_json::to_value(&pf).expect("serialize portfolio");
    assert!(value.get("commissions").is_some());
    assert!(value["positions"][0].get("status").is_some());
    let again: Portfolio = serde_json::from_value(value).expect("round-trip portfolio");
    assert_eq!(pf, again);
}

/// DX-COMP-001
#[test]
fn dx_comp_001_legacy_position_without_status_deserializes() {
    // decimalx wire v1：数量/价格为 {mantissa, scale}
    let json = r#"{
      "positionId":"p","instrument":{"exchange":"binance","symbol":"ETHUSDT"},"direction":"short",
      "quantity":{"mantissa":2,"scale":0},"entryPrice":{"mantissa":3000,"scale":0},
      "currentPrice":{"mantissa":2900,"scale":0},
      "unrealizedPnl":{"mantissa":200,"scale":0},"realizedPnl":{"mantissa":0,"scale":0},
      "createdAt":1700000000000,"updatedAt":1700000000000
    }"#;
    let pos: domainx::Position = serde_json::from_str(json).expect("legacy position");
    assert_eq!(pos.status, None);
    assert_eq!(pos.direction, domainx::PositionDirection::Short);
}

/// DX-COMP-001
#[test]
fn dx_comp_001_aggregate_commissions_empty() {
    assert!(aggregate_commissions_by_asset(&[]).expect("empty").is_empty());
    let one = [Commission { amount: Decimal::new(1, 0), asset: "BTC".into() }];
    assert_eq!(aggregate_commissions_by_asset(&one).expect("one").len(), 1);
}

/// DX-COMP-001：同资产 `Decimal::MAX + 1` 必须返回 Quantity，不得 panic。
#[test]
fn dx_comp_001_aggregate_commissions_overflow() {
    let items = [
        Commission { amount: Decimal::MAX, asset: "USDT".into() },
        Commission { amount: Decimal::ONE, asset: "USDT".into() },
    ];
    let err = aggregate_commissions_by_asset(&items).expect_err("overflow must be Err");
    assert!(
        matches!(err, ValidationError::Quantity(..)),
        "溢出须为 ValidationError::Quantity，得到 {err:?}"
    );
}
