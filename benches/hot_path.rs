#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! domainx 热路径：validate_order + serde 往返 + 佣金汇总。
//!
//! 每轮使用不同 quantity / 佣金金额，禁止空转同一 fixture 当完成。
use std::hint::black_box;
use std::time::Instant;

use domainx::{
    Commission, Decimal, InstrumentKey, Order, OrderSide, OrderStatus, OrderType, TimeInForce,
    aggregate_commissions_by_asset, validate_order,
};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 2_000 } else { 80_000 }
}

fn make_order(i: u32) -> Order {
    let qty = Decimal::new(i128::from(i) + 1, 0);
    let filled = Decimal::new(i128::from(i % 3), 0);
    let remaining = qty.checked_sub(filled).expect("qty >= filled");
    Order {
        order_id: format!("ord-{i}"),
        instrument: InstrumentKey { exchange: "binance".into(), symbol: "BTCUSDT".into() },
        side: if i.is_multiple_of(2) { OrderSide::Buy } else { OrderSide::Sell },
        order_type: OrderType::Limit,
        status: OrderStatus::New,
        price: Some(Decimal::new(50_000 + i128::from(i % 100), 0)),
        stop_price: None,
        quantity: qty,
        filled_quantity: filled,
        remaining_quantity: remaining,
        avg_fill_price: None,
        time_in_force: TimeInForce::Gtc,
        created_at: 1_700_000_000_000,
        updated_at: 1_700_000_000_000 + i64::from(i),
        client_order_id: Some(format!("cli-{i}")),
    }
}

fn make_commissions(i: u32) -> [Commission; 3] {
    [
        Commission { amount: Decimal::new(i128::from(i) + 1, 4), asset: "BNB".into() },
        Commission { amount: Decimal::new(i128::from(i % 17) + 2, 4), asset: "BNB".into() },
        Commission { amount: Decimal::new(i128::from(i % 53) + 3, 2), asset: "USDT".into() },
    ]
}

fn main() {
    let n = iters();
    for i in 0..n.min(32) {
        let order = make_order(i);
        let _ = black_box(validate_order(&order));
        let json = serde_json::to_vec(&order).expect("warmup serde");
        let _ = black_box(serde_json::from_slice::<Order>(&json));
        let _ = black_box(aggregate_commissions_by_asset(&make_commissions(i)));
    }

    let start = Instant::now();
    let mut acc = 0i128;
    for i in 0..n {
        let order = make_order(i);
        validate_order(&order).expect("validate");
        acc = acc.wrapping_add(order.quantity.mantissa());
        black_box(&order);
    }
    let validate_elapsed = start.elapsed();

    let start = Instant::now();
    for i in 0..n {
        let order = make_order(i);
        let json = serde_json::to_vec(&order).expect("serialize");
        let back: Order = serde_json::from_slice(&json).expect("deserialize");
        acc = acc.wrapping_add(back.filled_quantity.mantissa());
        black_box(back);
    }
    let serde_elapsed = start.elapsed();

    let start = Instant::now();
    for i in 0..n {
        let items = make_commissions(i);
        let aggregated = aggregate_commissions_by_asset(&items).expect("aggregate");
        acc = acc.wrapping_add(i128::try_from(aggregated.len()).expect("len"));
        black_box(aggregated);
    }
    let aggregate_elapsed = start.elapsed();

    println!(
        "bench_domainx_validate_order: iters={n} total={validate_elapsed:?} per_iter={:?}",
        validate_elapsed / n
    );
    println!(
        "bench_domainx_serde_roundtrip: iters={n} total={serde_elapsed:?} per_iter={:?}",
        serde_elapsed / n
    );
    println!(
        "bench_domainx_aggregate: iters={n} total={aggregate_elapsed:?} per_iter={:?} acc={}",
        aggregate_elapsed / n,
        black_box(acc)
    );
}
