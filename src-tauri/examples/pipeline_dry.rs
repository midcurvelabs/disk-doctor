//! Prints what the Pipeline tab would offer, without touching anything.
//! `cargo run --example pipeline_dry`

fn gb(b: u64) -> f64 {
    b as f64 / 1_073_741_824.0
}

fn main() {
    let scan = disk_doctor_lib::pipeline::scan().expect("scan");
    println!(
        "backup {} mounted={} free={:.1} GB | trash {:.1} GB | threshold {} GB",
        scan.backup.root,
        scan.backup.mounted,
        gb(scan.backup.free_bytes),
        gb(scan.trash_bytes),
        scan.threshold_gb
    );
    for r in &scan.rules {
        println!(
            "\n[{}] tier {} {:?}  ready {:.2} GB  waiting {:.2} GB  {}",
            r.id,
            r.tier,
            r.action,
            gb(r.ready_bytes),
            gb(r.waiting_bytes),
            r.error.clone().unwrap_or_default()
        );
        for i in r.items.iter().take(12) {
            println!(
                "   {} {:<48} {:>8.1} MB  {:>4} files  {}",
                if i.eligible { "+" } else { "." },
                i.name.chars().take(48).collect::<String>(),
                i.size_bytes as f64 / 1_048_576.0,
                i.file_count,
                i.reason
            );
        }
        if r.items.len() > 12 {
            println!("   ... {} more", r.items.len() - 12);
        }
    }
}
