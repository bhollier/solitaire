use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use solitaire::variant::klondike::*;

pub fn bench_game_rules_deal_all(c: &mut Criterion) {
    let deck: Deck = Card::new_deck();
    let initial_game_state = InitialGameState::from(deck);

    let mut group = c.benchmark_group("GameRules::deal_all");
    group.bench_function(BenchmarkId::new("deal_one", "Deck"), |b| {
        b.iter(|| {
            let mut game_state = initial_game_state.clone();
            loop {
                match GameRules::deal_one(game_state) {
                    DealResult::Dealing(new_state) => game_state = new_state,
                    DealResult::Complete(new_state) => return new_state,
                }
            }
        })
    });
    group.bench_function(BenchmarkId::new("deal_all", "_"), |b| {
        b.iter(|| GameRules::deal_all(initial_game_state.clone()))
    });
    group.finish();
}

criterion_group!(benches, bench_game_rules_deal_all);
criterion_main!(benches);
