//! Rust-only timing and profiling driver for the SolidCraft whole-operation cases
//! (`fixtures/solidcraft/*.json`, the same cases as `difftest --solidcraft-cases`).
//!
//! `cargo run --profile difftest -p difftest --example solidcraft_bench -- [options] [glob]`
//!
//! Options: `--fixes off|case|solidcraft|all` (default `case`: each case's own fixes;
//! `off`: none; `solidcraft`: `UpstreamFixes::solidcraft()` plus the case's own; `all`:
//! every `UpstreamFixes` field on), `--reps N` (runs per case, default 1; the reported
//! time is the minimum), `--dir D` (case files from `D` instead of
//! `fixtures/solidcraft/`), `--quiet`. Prints one line per case (verdict, F/E/V, volume
//! bits, best time) and the total, so two builds can be compared for identical results
//! and for speed. The driver times only; the library never reads a clock.

use std::path::Path;
use std::time::{Duration, Instant};

use difftest::solidcraft::{self, Run};
use solidcore::kernel::{Kernel, UpstreamFixes};

fn kernel(mode: &str) -> Kernel {
    let mut k = Kernel::default();
    match mode {
        "solidcraft" => k.fixes = UpstreamFixes::solidcraft(),
        "all" => {
            for name in UpstreamFixes::SOLIDCRAFT_FORK
                .iter()
                .chain(UpstreamFixes::SOLIDCRAFT_REPORTED)
                .chain(UpstreamFixes::NOT_SOLIDCRAFT)
            {
                k.fixes.set_by_name(name, true);
            }
        }
        _ => {}
    }
    k
}

fn summary(run: &Run) -> String {
    match run {
        Run::Done(m) => {
            let [f, e, v] = m.topology;
            format!(
                "done lumps={} F/E/V={f}/{e}/{v} volume={:e} ({:016x})",
                m.lumps,
                m.volume,
                m.volume.to_bits()
            )
        }
        Run::Deferred(d) => format!("deferred {}: {}", d.blocked_by, d.symbol),
        Run::Error(e) => format!("error {e}"),
    }
}

fn main() {
    let mut mode = "case".to_string();
    let mut reps = 1usize;
    let mut quiet = false;
    let mut glob = None;
    let mut dir = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--fixes" => mode = args.next().unwrap_or_default(),
            "--reps" => reps = args.next().and_then(|n| n.parse().ok()).unwrap_or(1),
            "--quiet" => quiet = true,
            "--dir" => dir = args.next().map(std::path::PathBuf::from),
            _ => glob = Some(a),
        }
    }
    if !["off", "case", "solidcraft", "all"].contains(&mode.as_str()) {
        eprintln!("--fixes must be off, case, solidcraft or all");
        std::process::exit(2);
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = dir.unwrap_or_else(|| solidcraft::cases_dir(&root));
    let cases = match solidcraft::load(&dir, glob.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    let mut total = Duration::ZERO;
    for l in &cases {
        let mut case = l.case.clone();
        if mode == "off" {
            case.fixes.clear();
        }
        let mut best = Duration::MAX;
        let mut out = String::new();
        for _ in 0..reps.max(1) {
            let t0 = Instant::now();
            let run = solidcraft::run_rust_on(kernel(&mode), &case);
            best = best.min(t0.elapsed());
            out = summary(&run);
        }
        total += best;
        if !quiet {
            println!("{:<64} {:>10.3} ms  {out}", l.id, best.as_secs_f64() * 1e3);
        }
    }
    println!(
        "SOLIDCRAFT-BENCH fixes={mode} cases={} total={:.3} ms",
        cases.len(),
        total.as_secs_f64() * 1e3
    );
}
