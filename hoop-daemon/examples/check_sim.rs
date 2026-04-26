use hoop_daemon::embedding::{cosine_similarity, jaccard_similarity, Embedder, NgramEmbedder};

fn main() {
    let embedder = NgramEmbedder::new();

    let pairs = vec![
        (
            "Set up CI/CD pipeline for deploys",
            "Configure continuous deployment pipeline",
        ),
        (
            "Implement caching layer with Redis",
            "Add Redis caching for performance",
        ),
        (
            "Add user model CRUD operations",
            "Implement CRUD for user model",
        ),
        (
            "Fix async task queue deadlock",
            "Repair async queue deadlock issue",
        ),
        (
            "Implement RPC error handling",
            "Add error handling for RPC calls",
        ),
    ];

    println!("{:<50} | Cosine | Jaccard | Max   | Boosted", "Pair");
    println!("{}", "-".repeat(95));

    for (a, b) in &pairs {
        let emb_a = embedder.embed(a);
        let emb_b = embedder.embed(b);
        let cosine = cosine_similarity(&emb_a, &emb_b);

        let tokens_a = embedder.canonical_tokens(a);
        let tokens_b = embedder.canonical_tokens(b);
        let jaccard = jaccard_similarity(&tokens_a, &tokens_b);

        let base = cosine.max(jaccard);
        let boost = if cosine > 0.65 && jaccard > 0.65 {
            0.05 * cosine.min(jaccard)
        } else {
            0.0
        };
        let boosted = base + boost;
        println!(
            "{:<25} vs {:<24} | {:.3}   | {:.3}    | {:.3}  | {:.3}",
            a, b, cosine, jaccard, base, boosted
        );
    }
}
