use std::time::Duration;
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use matching_engine::{MatchingEngine, Order, Side, OrderType, Price, Quantity, OrderId, Timestamp};

fn order(id: u64, side: Side, order_type: OrderType, price: i64, qty: u64) -> Order {
    Order {
        id: OrderId(id),
        side,
        order_type,
        price: Price(price),
        quantity: Quantity(qty),
        timestamp: Timestamp(id),
    }
}

// ─── 浅い板（比較ベースライン） ──────────────────────────────────────────────

fn bench_limit_resting(c: &mut Criterion) {
    c.bench_function("limit_resting", |b| {
        b.iter_batched(
            || {
                let mut e = MatchingEngine::new();
                e.submit(order(1, Side::Sell, OrderType::Limit, 5_010_000, 100));
                e
            },
            |mut e| e.submit(order(2, Side::Buy, OrderType::Limit, 5_000_000, 10)),
            BatchSize::SmallInput,
        )
    });
}

fn bench_full_match(c: &mut Criterion) {
    c.bench_function("full_match", |b| {
        b.iter_batched(
            || {
                let mut e = MatchingEngine::new();
                e.submit(order(1, Side::Sell, OrderType::Limit, 5_000_000, 100));
                e
            },
            |mut e| e.submit(order(2, Side::Buy, OrderType::Limit, 5_000_000, 100)),
            BatchSize::SmallInput,
        )
    });
}

fn bench_sweep_10_levels(c: &mut Criterion) {
    c.bench_function("sweep_10_levels", |b| {
        b.iter_batched(
            || {
                let mut e = MatchingEngine::new();
                for i in 0..10u64 {
                    e.submit(order(i + 1, Side::Sell, OrderType::Limit, 5_000_000 + i as i64, 10));
                }
                e
            },
            |mut e| e.submit(order(11, Side::Buy, OrderType::Market, 0, 100)),
            BatchSize::SmallInput,
        )
    });
}

fn bench_cancel(c: &mut Criterion) {
    c.bench_function("cancel", |b| {
        b.iter_batched(
            || {
                let mut e = MatchingEngine::new();
                e.submit(order(1, Side::Sell, OrderType::Limit, 5_000_000, 100));
                e
            },
            |mut e| e.cancel(OrderId(1)),
            BatchSize::SmallInput,
        )
    });
}

// ─── 深い板（キャッシュ破壊） ────────────────────────────────────────────────
//
// 買い 10万件（4_000_001〜4_100_000）＋売り 10万件（6_000_001〜6_100_000）を
// 事前に積み、合計 ~50MB の BTreeMap / HashMap をキャッシュに乗り切らない状態に
// してから各操作を計測する。

const DEPTH: u64 = 100_000;

fn make_deep_engine() -> (MatchingEngine, u64) {
    let mut e = MatchingEngine::new();
    let mut id = 0u64;
    for i in 0..DEPTH {
        id += 1;
        e.submit(order(id, Side::Buy,  OrderType::Limit, 4_000_000 + i as i64, 100));
    }
    for i in 0..DEPTH {
        id += 1;
        e.submit(order(id, Side::Sell, OrderType::Limit, 6_000_000 + i as i64, 100));
    }
    (e, id)
}

// 指値 resting: スプレッド内（5M 付近）への BTreeMap 挿入コスト
fn bench_limit_resting_deep(c: &mut Criterion) {
    let (mut e, mut id) = make_deep_engine();
    c.bench_function("limit_resting_deep", |b| {
        b.iter(|| {
            id += 1;
            // 1000 価格をサイクルして既存ノードへの push_back と新規ノード挿入を混在させる
            let price = 5_000_000 - (id % 1_000) as i64;
            black_box(e.submit(order(id, Side::Buy, OrderType::Limit, price, 10)))
        })
    });
}

// 完全約定: iter_custom でセットアップ（売り1件追加）を計測外に置く
fn bench_full_match_deep(c: &mut Criterion) {
    let (mut e, mut id) = make_deep_engine();
    c.bench_function("full_match_deep", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            for _ in 0..iters {
                id += 1;
                e.submit(order(id, Side::Sell, OrderType::Limit, 5_000_000, 100));
                let start = std::time::Instant::now();
                id += 1;
                black_box(e.submit(order(id, Side::Buy, OrderType::Limit, 5_000_000, 100)));
                total += start.elapsed();
            }
            total
        })
    });
}

// 10 レベルスイープ: 計測前に消費分を補充、計測は sweep のみ
fn bench_sweep_deep(c: &mut Criterion) {
    let (mut e, mut id) = make_deep_engine();
    c.bench_function("sweep_10_levels_deep", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            for _ in 0..iters {
                for level in 0..10i64 {
                    id += 1;
                    e.submit(order(id, Side::Sell, OrderType::Limit, 5_000_001 + level, 10));
                }
                let start = std::time::Instant::now();
                id += 1;
                black_box(e.submit(order(id, Side::Buy, OrderType::Market, 0, 100)));
                total += start.elapsed();
            }
            total
        })
    });
}

// キャンセル: 計測前に対象注文を追加、計測は cancel のみ
fn bench_cancel_deep(c: &mut Criterion) {
    let (mut e, mut id) = make_deep_engine();
    c.bench_function("cancel_deep", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            for _ in 0..iters {
                id += 1;
                e.submit(order(id, Side::Sell, OrderType::Limit, 5_500_000, 10));
                let cancel_id = OrderId(id);
                let start = std::time::Instant::now();
                black_box(e.cancel(cancel_id));
                total += start.elapsed();
            }
            total
        })
    });
}

criterion_group!(
    benches,
    bench_limit_resting,
    bench_full_match,
    bench_sweep_10_levels,
    bench_cancel,
    bench_limit_resting_deep,
    bench_full_match_deep,
    bench_sweep_deep,
    bench_cancel_deep,
);
criterion_main!(benches);
