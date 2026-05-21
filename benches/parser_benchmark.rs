use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use speechmarkdown_rust::SpeechMarkdownParser;

fn bench_parsing(c: &mut Criterion) {
    let parser = SpeechMarkdownParser;
    let inputs = vec![
        ("simple", "Hello world"),
        ("with_breaks", "Sample [3s] speech [250ms] markdown"),
        ("complex", "Why do you keep switching voices (from one)[voice:\"Brian\"] to (the other)[voice:\"Kendra\"]?"),
    ];

    for (name, input) in inputs {
        c.bench_with_input(BenchmarkId::new("parse", name), input, |b, i| {
            b.iter(|| parser.parse(black_box(i)));
        });
    }
}

criterion_group!(benches, bench_parsing);
criterion_main!(benches);