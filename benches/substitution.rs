use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;
use varsubst::substitute;

fn bench_single_variable(c: &mut Criterion) {
    let mut vars = HashMap::new();
    vars.insert("VAR", "value");

    c.bench_function("single variable", |b| {
        b.iter(|| substitute(black_box("Hello ${VAR}!"), black_box(&vars)))
    });
}

fn bench_multiple_variables(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiple_variables");

    for count in [5, 10, 20, 50, 100].iter() {
        let mut vars = HashMap::new();
        for i in 0..*count {
            vars.insert(format!("VAR{}", i).leak() as &str, "value");
        }

        let template = (0..*count)
            .map(|i| format!("${{VAR{}}}", i))
            .collect::<Vec<_>>()
            .join(" ");

        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, _| {
            b.iter(|| substitute(black_box(&template), black_box(&vars)))
        });
    }
    group.finish();
}

fn bench_fast_path_no_variables(c: &mut Criterion) {
    let vars: HashMap<&str, &str> = HashMap::new();

    c.bench_function("fast path (no variables)", |b| {
        b.iter(|| {
            substitute(
                black_box("This is a plain string with no variables at all"),
                black_box(&vars),
            )
        })
    });
}

fn bench_large_template(c: &mut Criterion) {
    let mut vars = HashMap::new();
    vars.insert("USER", "alice");
    vars.insert("HOME", "/home/alice");
    vars.insert("SHELL", "/bin/bash");

    // Create a large template (1KB)
    let template = "User: ${USER}, Home: ${HOME}, Shell: ${SHELL}\n".repeat(20);

    c.bench_function("large template (1KB)", |b| {
        b.iter(|| substitute(black_box(&template), black_box(&vars)))
    });
}

fn bench_many_lookups(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_performance");

    // Test with different numbers of variables in the map
    for map_size in [10, 50, 100, 500].iter() {
        let mut vars = HashMap::new();
        for i in 0..*map_size {
            vars.insert(format!("VAR{}", i).leak() as &str, "value");
        }

        // Template that references a few variables
        let template = "${VAR1} ${VAR5} ${VAR10}";

        group.bench_with_input(
            BenchmarkId::new("map_size", map_size),
            map_size,
            |b, _| b.iter(|| substitute(black_box(template), black_box(&vars))),
        );
    }
    group.finish();
}

fn bench_undefined_variables(c: &mut Criterion) {
    let mut vars = HashMap::new();
    vars.insert("DEFINED", "value");

    c.bench_function("undefined variables", |b| {
        b.iter(|| {
            substitute(
                black_box("${DEFINED} ${UNDEFINED1} ${UNDEFINED2} ${UNDEFINED3}"),
                black_box(&vars),
            )
        })
    });
}

fn bench_escape_sequences(c: &mut Criterion) {
    let vars: HashMap<&str, &str> = HashMap::new();

    c.bench_function("escape sequences", |b| {
        b.iter(|| {
            substitute(
                black_box(r"Escaped: \${VAR} \${ANOTHER} normal text"),
                black_box(&vars),
            )
        })
    });
}

fn bench_real_world_template(c: &mut Criterion) {
    let mut vars = HashMap::new();
    vars.insert("APP_NAME", "MyApp");
    vars.insert("VERSION", "1.0.0");
    vars.insert("ENV", "production");
    vars.insert("DB_HOST", "localhost");
    vars.insert("DB_PORT", "5432");
    vars.insert("DB_NAME", "mydb");
    vars.insert("SERVER_HOST", "0.0.0.0");
    vars.insert("SERVER_PORT", "8080");

    let template = r#"
    Application: ${APP_NAME} v${VERSION}
    Environment: ${ENV}

    Database:
      Host: ${DB_HOST}
      Port: ${DB_PORT}
      Database: ${DB_NAME}

    Server:
      Host: ${SERVER_HOST}
      Port: ${SERVER_PORT}
    "#;

    c.bench_function("real world template", |b| {
        b.iter(|| substitute(black_box(template), black_box(&vars)))
    });
}

criterion_group!(
    benches,
    bench_single_variable,
    bench_multiple_variables,
    bench_fast_path_no_variables,
    bench_large_template,
    bench_many_lookups,
    bench_undefined_variables,
    bench_escape_sequences,
    bench_real_world_template
);
criterion_main!(benches);
