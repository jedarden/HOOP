use hoop_daemon::embedding::{Embedder, NgramEmbedder};

fn main() {
    let embedder = NgramEmbedder::new();

    let texts = [
        "Set up CI/CD pipeline for deploys",
        "Configure continuous deployment pipeline",
        "Implement caching layer with Redis",
        "Add Redis caching for performance",
    ];

    for text in texts {
        let tokens = embedder.canonical_tokens(text);
        println!("'{}'", text);
        println!("  Tokens: {:?}", tokens);
        println!();
    }
}
