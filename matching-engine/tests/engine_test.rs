use matching_engine::*;

fn order(id: u64, side: Side, ot: OrderType, price: i64, qty: u64, ts: u64) -> Order {
    Order {
        id: OrderId(id),
        side,
        order_type: ot,
        price: Price(price),
        quantity: Quantity(qty),
        timestamp: Timestamp(ts),
    }
}

#[test]
//指値が板に残る場合
fn limit_resting() {
    let mut e = MatchingEngine::new();
    let r = e.submit(order(1, Side::Buy, OrderType::Limit, 5_000_000, 100, 1));
    assert_eq!(r.status, ExecStatus::Resting);
    assert_eq!(e.book().best_bid(), Some(Price(5_000_000)));
}

#[test]
//同じ価格、同じ注文量の場合
fn full_match_at_same_price() {
    let mut e = MatchingEngine::new();
    e.submit(order(1, Side::Sell, OrderType::Limit, 5_000_000, 100, 1));
    let r = e.submit(order(2, Side::Buy, OrderType::Limit, 5_000_000, 100, 2));
    assert_eq!(r.status, ExecStatus::Filled);
    assert_eq!(r.trades.len(), 1);
    assert_eq!(r.trades[0].price, Price(5_000_000));
}

#[test]
//注文の順番　二つの注文が並んでいる場合
fn time_priority_within_level() {
    let mut e = MatchingEngine::new();
    e.submit(order(1, Side::Sell, OrderType::Limit, 5_000_000, 100, 1));
    e.submit(order(2, Side::Sell, OrderType::Limit, 5_000_000, 100, 2));
    let r = e.submit(order(3, Side::Buy, OrderType::Limit, 5_000_000, 100, 3));
    assert_eq!(r.trades[0].maker_order_id, OrderId(1));
}

#[test]
//テイカー注文が複数のメイカー注文を食う場合
fn sweep_multiple_levels() {
    let mut e = MatchingEngine::new();
    e.submit(order(1, Side::Sell, OrderType::Limit, 5_000_000, 50, 1));
    e.submit(order(2, Side::Sell, OrderType::Limit, 5_000_100, 50, 2));
    let r = e.submit(order(3, Side::Buy, OrderType::Limit, 5_000_200, 100, 3));
    assert_eq!(r.status, ExecStatus::Filled);
    assert_eq!(r.trades.len(), 2);
}

#[test]
//削除テスト
fn cancel_works() {
    let mut e = MatchingEngine::new();
    e.submit(order(1, Side::Buy, OrderType::Limit, 5_000_000, 100, 1));
    let canceled = e.cancel(OrderId(1));
    assert!(canceled.is_some());
    assert_eq!(e.book().best_bid(), None);
}

