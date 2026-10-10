use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusche::Evaluator;

fn with_prelude(src: &str) -> Evaluator {
    let e = Evaluator::with_prelude();
    e.eval_str(src).expect("setup failed");
    e
}

fn bench_fib(c: &mut Criterion) {
    let e = with_prelude(
        r#"
        (define (fib n)
            (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2)))))
        "#,
    );
    c.bench_function("fib 25", |b| {
        b.iter(|| {
            let r = e.eval_str("(fib 25)").unwrap();
            black_box(r);
        })
    });
}

fn bench_count_down(c: &mut Criterion) {
    let e = with_prelude(
        r#"
        (define (count-down n)
            (if (= n 0) 'done (count-down (- n 1))))
        "#,
    );
    c.bench_function("count-down 1e6", |b| {
        b.iter(|| {
            let r = e.eval_str("(count-down 1000000)").unwrap();
            black_box(r);
        })
    });
}

fn bench_mandelbrot(c: &mut Criterion) {
    // Kernel from examples/mandelbrot.rsc without I/O; 24x8 grid, max-iter 20.
    let e = with_prelude(
        r#"
        (define max-iter 20)
        (define (escape-count cr ci)
          (define (iter zr zi n)
            (let ((zr2 (* zr zr))
                  (zi2 (* zi zi)))
              (cond ((= n max-iter) n)
                    ((> (+ zr2 zi2) 4) n)
                    (else (iter (+ (- zr2 zi2) cr)
                                (+ (* 2 zr zi) ci)
                                (+ n 1))))))
          (iter 0 0 0))
        (define (render width height)
          (let ((dx (/ 2.8 width))
                (dy (/ 2.4 height))
                (row 0) (sum 0))
            (while (< row height)
              (let ((ci (+ -1.2 (* row dy)))
                    (col 0))
                (while (< col width)
                  (set! sum (+ sum (escape-count (+ -2.1 (* col dx)) ci)))
                  (set! col (+ col 1))))
              (set! row (+ row 1)))
            sum))
        "#
    );
    c.bench_function("mandelbrot 24x8", |b| {
        b.iter(|| {
            let r = e.eval_str("(render 24 8)").unwrap();
            black_box(r);
        })
    });
}

fn bench_macro_heavy(c: &mut Criterion) {
    let e = with_prelude(
        r#"
        (define (score n)
          (cond ((= n 0) 0)
                ((< n 0) -1)
                (else (let ((a (+ n 1)) (b (- n 1)))
                        (+ a b)))))
        (define (loop n acc)
          (if (= n 0) acc (loop (- n 1) (+ acc (score n)))))
        "#,
    );
    c.bench_function("macro-heavy 10000", |b| {
        b.iter(|| {
            let r = e.eval_str("(loop 10000 0)").unwrap();
            black_box(r);
        })
    });
}

fn bench_list_ops(c: &mut Criterion) {
    let e = with_prelude(
        r#"
        (define (range n)
          (define (loop i acc)
            (if (= i 0) acc (loop (- i 1) (cons i acc))))
          (loop n '()))
        (define nums (range 10000))
        "#,
    );
    c.bench_function("list-ops map/filter/fold 10k", |b| {
        b.iter(|| {
            let r = e
                .eval_str(
                    r#"
                    (fold + 0
                      (map (lambda (x) (* x 2))
                        (filter (lambda (x) (= (% x 2) 0)) nums)))
                    "#,
                )
                .unwrap();
            black_box(r);
        })
    });
}

fn bench_startup(c: &mut Criterion) {
    c.bench_function("startup Evaluator::default", |b| {
        b.iter(|| {
            let e = Evaluator::default();
            black_box(e);
        })
    });
}

fn bench_lookup_deep(c: &mut Criterion) {
    let e = with_prelude(
        r#"
        (define deep-var 42)
        (define (lookup-deep)
          ((lambda (a)
            ((lambda (b)
              ((lambda (c)
                ((lambda (d)
                  ((lambda (e)
                    ((lambda (f)
                      ((lambda (g)
                        ((lambda (h)
                          ((lambda (i)
                            ((lambda (j) deep-var)
                             10))
                           9))
                         8))
                       7))
                     6))
                   5))
                 4))
               3))
             2))
           1))
        "#,
    );
    c.bench_function("lookup-deep 1e5", |b| {
        b.iter(|| {
            let r = e
                .eval_str(
                    r#"
                    (define (loop n acc)
                      (if (= n 0) acc (loop (- n 1) (+ acc (lookup-deep)))))
                    (loop 100000 0)
                    "#,
                )
                .unwrap();
            black_box(r);
        })
    });
}

criterion_group!(
    benches,
    bench_fib,
    bench_count_down,
    bench_mandelbrot,
    bench_macro_heavy,
    bench_list_ops,
    bench_startup,
    bench_lookup_deep,
);
criterion_main!(benches);
