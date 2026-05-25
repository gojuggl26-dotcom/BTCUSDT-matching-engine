//! MatchingEngine の統合テスト

mod common;

use common::{limit_order, market_order};
use matching_engine::{ExecStatus, MatchingEngine, OrderType, Side};

#[test]
fn side_opposite_unit_smoke() {
    assert_eq!(Side::Buy.opposite(), Side::Sell);
}

#[test]
#[ignore = "engine / orderbook の実装がコンパイル可能になったら外す"]
fn market_buy_matches_resting_sell() {
    let mut engine = MatchingEngine::new();

    let sell = limit_order(1, Side::Sell, 5_010_000, 50_000_000, 1);
    let r = engine.submit(sell);
    assert_eq!(r.status, ExecStatus::Resting);

    let buy = market_order(2, Side::Buy, 30_000_000, 2);
    let r = engine.submit(buy);
    assert!(!r.trades.is_empty());
    assert_eq!(r.trades[0].quantity.0, 30_000_000);
}

#[test]
#[ignore = "engine / orderbook の実装がコンパイル可能になったら外す"]
fn limit_buy_does_not_cross_low_ask() {
    let mut engine = MatchingEngine::new();

    engine.submit(limit_order(1, Side::Sell, 5_010_000, 10, 1));
    let r = engine.submit(limit_order(2, Side::Buy, 5_000_000, 5, 2));

    assert_eq!(r.status, ExecStatus::Resting);
    assert!(r.trades.is_empty());
}
