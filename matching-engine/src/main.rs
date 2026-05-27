use matching_engine::*;

//テストコードのためあらかじめ引数などはランダムな値を代入してある
fn main() {
    let mut engine = MatchingEngine::new();

    //売り注文　ここでは0.5BTCとする
    let sell1 = Order {
        id: OrderId(1),
        side: Side::Sell,
        order_type: OrderType::Limit,
        price: Price(5_010_000),
        quantity: Quantity(50_000_000),
        timestamp: Timestamp(1),
    };

    let r = engine.submit(sell1);
    println!("売り注文１：{:?}", r.status);
    //買い注文　ここでは成り行き　0.3BTC
    let buy_mkt = Order {
        id: OrderId(2),
        side: Side::Buy,
        order_type: OrderType::Market,
        price: Price(0),
        quantity: Quantity(30_000_000),
        timestamp: Timestamp(2),
    };
    let r = engine.submit(buy_mkt);
    println!("買い注文(メイカー): {:?}", r);

    println!("best_bid = {:?}", engine.book().best_bid());
    println!("best_ask = {:?}", engine.book().best_ask());

}

