// Comparison harness for Fusabi vs Rhai vs Lua
// Benchmarks the same algorithms across different embedded scripting languages

use std::time::Duration;

mod benchmarks;

fn print_header() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║        Fusabi Benchmark Comparison Suite                      ║");
    println!("║        Fusabi vs Rhai vs Lua                                  ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
}

fn print_benchmark_header(name: &str) {
    println!("\n┌────────────────────────────────────────────────────────────────┐");
    println!("│ Benchmark: {:<52} │", name);
    println!("└────────────────────────────────────────────────────────────────┘");
}

fn format_duration(duration: Duration) -> String {
    let nanos = duration.as_nanos();
    if nanos < 1_000 {
        format!("{} ns", nanos)
    } else if nanos < 1_000_000 {
        format!("{:.2} µs", nanos as f64 / 1_000.0)
    } else if nanos < 1_000_000_000 {
        format!("{:.2} ms", nanos as f64 / 1_000_000.0)
    } else {
        format!("{:.2} s", nanos as f64 / 1_000_000_000.0)
    }
}

fn run_benchmark<F>(_name: &str, mut f: F) -> Duration
where
    F: FnMut(),
{
    // Warmup
    for _ in 0..3 {
        f();
    }

    // Measure
    let iterations = 10;
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        f();
    }
    let total = start.elapsed();
    total / iterations
}

fn print_results(fusabi_time: Duration, rhai_time: Duration, lua_time: Duration) {
    let fusabi_nanos = fusabi_time.as_nanos() as f64;
    let rhai_nanos = rhai_time.as_nanos() as f64;
    let lua_nanos = lua_time.as_nanos() as f64;

    println!("\nResults:");
    println!("  Fusabi: {}", format_duration(fusabi_time));
    println!("  Rhai:   {}", format_duration(rhai_time));
    println!("  Lua:    {}", format_duration(lua_time));

    println!("\nRelative Performance (Fusabi = 1.00x):");
    println!("  Fusabi: 1.00x");
    println!("  Rhai:   {:.2}x", rhai_nanos / fusabi_nanos);
    println!("  Lua:    {:.2}x", lua_nanos / fusabi_nanos);

    // Determine winner
    let min_time = fusabi_nanos.min(rhai_nanos).min(lua_nanos);
    let winner = if (fusabi_nanos - min_time).abs() < 0.001 {
        "Fusabi"
    } else if (rhai_nanos - min_time).abs() < 0.001 {
        "Rhai"
    } else {
        "Lua"
    };
    println!("\n  Winner: {}", winner);
}

fn main() {
    print_header();

    // Fibonacci benchmark
    print_benchmark_header("Recursive Fibonacci (fib 20)");
    let fusabi_fib = run_benchmark("fusabi_fib", || {
        benchmarks::fusabi_fibonacci();
    });
    let rhai_fib = run_benchmark("rhai_fib", || {
        benchmarks::rhai_fibonacci();
    });
    let lua_fib = run_benchmark("lua_fib", || {
        benchmarks::lua_fibonacci();
    });
    print_results(fusabi_fib, rhai_fib, lua_fib);

    // Sieve benchmark
    print_benchmark_header("Sieve of Eratosthenes (primes up to 100)");
    let fusabi_sieve = run_benchmark("fusabi_sieve", || {
        benchmarks::fusabi_sieve();
    });
    let rhai_sieve = run_benchmark("rhai_sieve", || {
        benchmarks::rhai_sieve();
    });
    let lua_sieve = run_benchmark("lua_sieve", || {
        benchmarks::lua_sieve();
    });
    print_results(fusabi_sieve, rhai_sieve, lua_sieve);

    // Ackermann benchmark
    print_benchmark_header("Ackermann Function (ack 3 7)");
    let fusabi_ack = run_benchmark("fusabi_ack", || {
        benchmarks::fusabi_ackermann();
    });
    let rhai_ack = run_benchmark("rhai_ack", || {
        benchmarks::rhai_ackermann();
    });
    let lua_ack = run_benchmark("lua_ack", || {
        benchmarks::lua_ackermann();
    });
    print_results(fusabi_ack, rhai_ack, lua_ack);

    // Array operations benchmark
    print_benchmark_header("Array Operations (create and sum 1000 elements)");
    let fusabi_array = run_benchmark("fusabi_array", || {
        benchmarks::fusabi_array_ops();
    });
    let rhai_array = run_benchmark("rhai_array", || {
        benchmarks::rhai_array_ops();
    });
    let lua_array = run_benchmark("lua_array", || {
        benchmarks::lua_array_ops();
    });
    print_results(fusabi_array, rhai_array, lua_array);

    // Summary table
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                     Summary Table                             ║");
    println!("╠════════════════════════════════╦═══════════╦═══════════╦═══════════╣");
    println!("║ Benchmark                      ║  Fusabi   ║   Rhai    ║    Lua    ║");
    println!("╠════════════════════════════════╬═══════════╬═══════════╬═══════════╣");
    println!(
        "║ Fibonacci (fib 20)             ║ {:>9} ║ {:>9} ║ {:>9} ║",
        format_duration(fusabi_fib),
        format_duration(rhai_fib),
        format_duration(lua_fib)
    );
    println!(
        "║ Sieve (primes to 100)          ║ {:>9} ║ {:>9} ║ {:>9} ║",
        format_duration(fusabi_sieve),
        format_duration(rhai_sieve),
        format_duration(lua_sieve)
    );
    println!(
        "║ Ackermann (3, 7)               ║ {:>9} ║ {:>9} ║ {:>9} ║",
        format_duration(fusabi_ack),
        format_duration(rhai_ack),
        format_duration(lua_ack)
    );
    println!(
        "║ Array Ops (1000 elements)      ║ {:>9} ║ {:>9} ║ {:>9} ║",
        format_duration(fusabi_array),
        format_duration(rhai_array),
        format_duration(lua_array)
    );
    println!("╚════════════════════════════════╩═══════════╩═══════════╩═══════════╝\n");
}
