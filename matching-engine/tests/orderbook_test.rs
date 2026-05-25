//! OrderBook の統合テスト

mod common;

use common::limit_order;
use matching_engine::{OrderBook, OrderId, Side};

#[test]
#[ignore = "orderbook の実装がコンパイル可能になったら外す"]
fn add_and_cancel_order() {
    let mut book = OrderBook::new();
    let order = limit_order(1, Side::Sell, 5_010_000, 100, 1);

    book.add(order.clone());
    assert_eq!(book.best_ask(), Some(order.price));

    let removed = book.cancel(OrderId(1)).expect("order should exist");
    assert_eq!(removed.id, OrderId(1));
    assert_eq!(book.best_ask(), None);
}

#[test]
#[ignore = "orderbook の実装がコンパイル可能になったら外す"]
fn best_bid_is_highest_price() {
    let mut book = OrderBook::new();
    book.add(limit_order(1, Side::Buy, 4_990_000, 10, 1));
    book.add(limit_order(2, Side::Buy, 5_000_000, 10, 2));

    assert_eq!(book.best_bid(), Some(matching_engine::Price(5_000_000)));
}
