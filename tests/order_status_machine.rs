#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! T-STATE-MACHINE / T-BACKTEST：测试内私有合法/非法转移表。
//!
//! 禁止生产 `is_legal_*` 公共 API；本文件函数均为测试私有。

use domainx::OrderStatus;
use serde::Deserialize;

/// 测试内私有合法边。New→Partial→Filled 为最小合法链。
fn legal_edges() -> &'static [(OrderStatus, OrderStatus)] {
    &[
        (OrderStatus::New, OrderStatus::PartiallyFilled),
        (OrderStatus::PartiallyFilled, OrderStatus::Filled),
    ]
}

/// 测试内私有非法边（至少 3 条）。
fn illegal_edges() -> &'static [(OrderStatus, OrderStatus)] {
    &[
        (OrderStatus::Filled, OrderStatus::New),
        (OrderStatus::Canceled, OrderStatus::PartiallyFilled),
        (OrderStatus::Rejected, OrderStatus::Filled),
    ]
}

fn table_allows(from: &OrderStatus, to: &OrderStatus) -> bool {
    legal_edges().iter().any(|(a, b)| a == from && b == to)
}

fn parse_status(wire: &str) -> OrderStatus {
    serde_json::from_value(serde_json::Value::String(wire.to_string()))
        .unwrap_or_else(|e| panic!("无法解析 OrderStatus wire `{wire}`: {e}"))
}

#[derive(Debug, Deserialize)]
struct StatusSequenceFixture {
    legal: Vec<(String, String)>,
    illegal: Vec<(String, String)>,
}

#[test]
fn legal_chain_new_partial_filled() {
    assert!(
        table_allows(&OrderStatus::New, &OrderStatus::PartiallyFilled),
        "New→PartiallyFilled 必须合法"
    );
    assert!(
        table_allows(&OrderStatus::PartiallyFilled, &OrderStatus::Filled),
        "PartiallyFilled→Filled 必须合法"
    );
}

#[test]
fn illegal_edges_are_rejected() {
    assert!(illegal_edges().len() >= 3, "非法表至少 3 条，实际 {}", illegal_edges().len());
    for (from, to) in illegal_edges() {
        assert!(!table_allows(from, to), "非法边 {from:?}→{to:?} 不得出现在合法表");
    }
}

/// T-BACKTEST：夹具状态序列必须与测试内私有表一致。
#[test]
fn fixture_status_sequence_matches_private_table() {
    let raw = include_str!("fixtures/order_status_sequence.json");
    let fixture: StatusSequenceFixture =
        serde_json::from_str(raw).expect("status sequence fixture");

    assert_eq!(fixture.legal.len(), legal_edges().len(), "合法边数量须与私有表一致");
    assert!(fixture.illegal.len() >= 3, "夹具非法边至少 3 条");

    for (from_wire, to_wire) in &fixture.legal {
        let from = parse_status(from_wire);
        let to = parse_status(to_wire);
        assert!(table_allows(&from, &to), "夹具合法边 {from_wire}→{to_wire} 不在私有表");
    }
    for (from_wire, to_wire) in &fixture.illegal {
        let from = parse_status(from_wire);
        let to = parse_status(to_wire);
        assert!(!table_allows(&from, &to), "夹具非法边 {from_wire}→{to_wire} 误入私有合法表");
    }
}
