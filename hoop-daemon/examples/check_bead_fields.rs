use std::fs::File;
use std::io::BufRead;

fn main() {
    let file = File::open("/home/coding/HOOP/testrepo/.beads/issues.jsonl").unwrap();
    let reader = std::io::BufReader::new(file);

    for (idx, line) in reader.lines().enumerate() {
        if let Ok(line) = line {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                println!("Line {}: {}", idx + 1, json.as_object().map(|obj| {
                    obj.keys().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")
                }).unwrap_or_else(|| "not an object".to_string()));
            } else {
                eprintln!("Line {}: Failed to parse JSON", idx + 1);
            }
        }
    }
}
