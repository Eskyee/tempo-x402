use x402_soul::opus_bench::load_embedded_problems;

fn main() {
    let problems = load_embedded_problems();
    for p in &problems {
        let j = serde_json::json!({
            "slug": p.slug,
            "difficulty": p.difficulty,
            "instructions": p.instructions,
            "starter_code": p.starter_code,
            "test_code": p.test_code,
            "cargo_toml": p.cargo_toml,
        });
        println!("{}", serde_json::to_string(&j).unwrap());
    }
}
