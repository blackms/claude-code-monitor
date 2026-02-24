use std::fs;
use claude_code_monitor::data::{models::StatsCache, calculations::{calculate_total_cost, ModelPricing}};

fn main() {
    let content = fs::read_to_string("/Users/alessiorocchi/.claude/stats-cache.json").unwrap();
    let stats: StatsCache = serde_json::from_str(&content).unwrap();
    let total = calculate_total_cost(&stats);
    println!("Total cost calculated: ${:.2}", total);
    
    for (model, usage) in &stats.model_usage {
        let p = ModelPricing::for_model(model);
        let mut t = 0.0;
        t += (usage.input_tokens as f64 / 1_000_000.0) * p.input;
        t += (usage.output_tokens as f64 / 1_000_000.0) * p.output;
        t += (usage.cache_read_input_tokens as f64 / 1_000_000.0) * p.cache_read;
        t += (usage.cache_creation_input_tokens as f64 / 1_000_000.0) * p.cache_create;
        println!("  - {}: ${:.2} (pricing: in={}, out={}, cr={}, cc={})", model, t, p.input, p.output, p.cache_read, p.cache_create);
    }
}
