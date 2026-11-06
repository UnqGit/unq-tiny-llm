use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};
use unq_tiny_llm::{
    loader::{
        DataLoadResult,
        load_data
    },
    tokenizer::{
        initial::{
            build_initial_token_sequence,
            initial_token_sequence,
            initial_tokens
        },
        preprocessing::preprocess_text
    }
};

fn tokenizer_init_benchmark(c: &mut Criterion) {
    let text = load_data("data/dataset/books/fiction/silmarillion/chapter_1.txt").expect("Failed to load data for benchmarking.");

    let DataLoadResult { contents, .. } = text;
    let base_content = contents.join("\n\0\n");
    let pretokens = preprocess_text(&base_content);

    let mut group = c.benchmark_group("Tokenizers Initialization");

    group.bench_function("Naive", |b| {
        b.iter(|| {
            // Generates Vec<Vec<TokenId>> that the naive implementation of BPE tokenizer works on.
            let _ = initial_token_sequence(black_box(&pretokens));
        });
    });

    group.bench_function("Min alloc", |b| {
        b.iter( || {
            // generates TokenArena struct which contains Vec<TokenId> and Vec<Range<usize>> as chunk metadata.
            let mut token_arena = initial_tokens(black_box(&pretokens));
            // generates the actual Vec<&mut [TokenId]> that min_alloc implementation of tokenizer works on.
            let _ = build_initial_token_sequence(black_box(&mut token_arena));
        });
    });
}

criterion_group!(benches, tokenizer_init_benchmark);
criterion_main!(benches);