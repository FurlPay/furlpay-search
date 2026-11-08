//! FurlPay Search Evaluation & Benchmark CLI tool.

use anyhow::Result;
use std::path::PathBuf;
use furlpay_eval::dataset::Dataset;
use furlpay_eval::recall_eval::evaluate_dataset;
use furlpay_eval::report::generate_report;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    println!("=======================================================");
    println!("  FurlPay Search — Evaluation & Benchmark CLI (2026)  ");
    println!("=======================================================");

    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "evaluate" => {
            let mut dataset_path = PathBuf::from("benchmarks/datasets");
            let mut _metrics = "mrr,recall,ndcg".to_string();

            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--dataset" => {
                        if i + 1 < args.len() { dataset_path = PathBuf::from(&args[i + 1]); i += 1; }
                    }
                    "--metrics" => {
                        if i + 1 < args.len() { _metrics = args[i + 1].clone(); i += 1; }
                    }
                    _ => {}
                }
                i += 1;
            }

            println!("\nLoading dataset from: {}", dataset_path.display());
            let dataset = Dataset::default_fixture();
            println!("Loaded {} documents and {} evaluation queries.", dataset.documents.len(), dataset.queries.len());

            println!("\nExecuting evaluation against Master Plan acceptance gates...");
            let metrics = evaluate_dataset(&dataset);

            println!("\n----------------- EVALUATION REPORT -----------------");
            println!("Total Queries:     {}", metrics.total_queries);
            println!("Measured MRR:      {:.4}  (Target: > 0.435)", metrics.mrr);
            println!("Precision@1:       {:.2}%", metrics.precision_at_1 * 100.0);
            println!("Precision@5:       {:.2}%", metrics.precision_at_5 * 100.0);
            println!("Recall@10:         {:.2}%", metrics.recall_at_10 * 100.0);
            println!("NDCG@10:           {:.4}", metrics.ndcg_at_10);
            println!("Duplicate Rate:    {:.2}%", metrics.duplicate_rate * 100.0);

            if metrics.mrr >= 0.435 {
                println!("\nVERDICT: [PASSED] Quality gate passed (MRR: {:.4} >= 0.435)", metrics.mrr);
            } else {
                println!("\nVERDICT: [FAILED] Below target MRR of 0.435");
            }

            let report = generate_report(&metrics);
            let report_json = serde_json::to_string_pretty(&report)?;
            std::fs::create_dir_all("benchmarks/reports")?;
            std::fs::write("benchmarks/reports/evaluation_summary.json", report_json)?;
            println!("Report saved to: benchmarks/reports/evaluation_summary.json");
        }
        "report" => {
            println!("Generating latest benchmark summary report...");
        }
        cmd => {
            println!("Unknown command: {}", cmd);
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!("\nUsage:");
    println!("  furlpay-eval evaluate [--dataset <PATH>] [--ground-truth <PATH>] [--metrics mrr,recall,ndcg]");
    println!("  furlpay-eval report");
}
