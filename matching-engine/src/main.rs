use matching_engine::*;

fn main() {
    let mut engine = MatchingEngine::new();

    let sell1 = Order {
        id: OrderId(1),
        side: Side::Sell,
        order_type: OrderType::Limit,
        price: Price(5_010_000),
        quantity: Quantity(50_000_000),
        timestamp: Timestamp(1),
    };

    let trades = engine.submit(sell1);
    println!("売り注文１：trades={}", trades.len());

    let buy_mkt = Order {
        id: OrderId(2),
        side: Side::Buy,
        order_type: OrderType::Market,
        price: Price(0),
        quantity: Quantity(30_000_000),
        timestamp: Timestamp(2),
    };
    let trades = engine.submit(buy_mkt);
    println!("買い注文(メイカー): {} trades", trades.len());

    println!("best_bid = {:?}", engine.book().best_bid());
    println!("best_ask = {:?}", engine.book().best_ask());
}
