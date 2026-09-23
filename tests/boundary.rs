#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! T-BOUNDARY：数量溢出能红；空 InstrumentKey 只记录现状（ADR-001 不做校验）。

use domainx::{
    Decimal, DecimalError, InstrumentKey, Order, OrderSide, OrderStatus, OrderType, TimeInForce,
    ValidationError, validate_order, validate_quantity_balance,
};
use std::error::Error;

fn valid_limit() -> Order {
    Order {
        order_id: "o1".into(),
        instrument: InstrumentKey { exchange: "binance".into(), symbol: "BTCUSDT".into() },
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        status: OrderStatus::New,
        price: Some(Decimal::new(50000, 0)),
        stop_price: None,
        quantity: Decimal::new(2, 0),
        filled_quantity: Decimal::new(1, 0),
        remaining_quantity: Decimal::new(1, 0),
        avg_fill_price: None,
        time_in_force: TimeInForce::Gtc,
        created_at: 1_700_000_000_000,
        updated_at: 1_700_000_001_000,
        client_order_id: None,
    }
}

/// filled + remaining 走 `Decimal::checked_add`；`MAX + 1` 必须 Quantity。
#[test]
fn quantity_balance_overflow_max_plus_one() {
    let err = validate_quantity_balance(Decimal::MAX, Decimal::MAX, Decimal::ONE)
        .expect_err("MAX + 1 必须溢出");
    assert!(
        matches!(err, ValidationError::Quantity(..)),
        "溢出须为 ValidationError::Quantity，得到 {err:?}"
    );
}

/// FOLLOW-UP-DX-001：溢出路径必须透传底层 `DecimalError` 的 `source()` 链。
#[test]
fn quantity_overflow_preserves_decimal_error_source() {
    let err = validate_quantity_balance(Decimal::MAX, Decimal::MAX, Decimal::ONE)
        .expect_err("MAX + 1 必须溢出");
    let src = err.source().expect("溢出须携带 source");
    let dec_err = src.downcast_ref::<DecimalError>().expect("source 须为 DecimalError");
    assert_eq!(dec_err, &DecimalError::RepresentationOverflow);
}

/// ADR-001：InstrumentKey 不做校验。空 exchange/symbol 不是 validate_order 失败原因。
#[test]
fn empty_instrument_key_is_not_validated_adr001() {
    let mut order = valid_limit();
    order.instrument = InstrumentKey { exchange: String::new(), symbol: String::new() };
    validate_order(&order).expect("空 InstrumentKey 按 ADR-001 不进入校验");
}
