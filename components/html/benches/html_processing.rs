use criterion::{black_box, criterion_group, criterion_main, Criterion};
use html::tokenizer::Tokenizer;
use html::tree_builder::TreeBuilder;

fn html_parsing_benchmark(c: &mut Criterion) {
    let html = include_str!("./purecss_gaze.html");
    c.bench_function("parse_purecss_gaze", |b| {
        b.iter(|| {
            let tokenizer = Tokenizer::new(black_box(html.chars()));
            let tree_builder = TreeBuilder::new(tokenizer);
            tree_builder.run();
        })
    });
}

criterion_group!(benches, html_parsing_benchmark);
criterion_main!(benches);
