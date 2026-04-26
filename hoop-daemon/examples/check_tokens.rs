use hoop_daemon::embedding::{Embedder, NgramEmbedder};

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
    ];

    for (a, b) in &pairs {
        let tokens_a = embedder.canonical_tokens(a);
        let tokens_b = embedder.canonical_tokens(b);
        println!("A: '{}'", a);
        println!("  Tokens: {:?}", tokens_a);
        println!("B: '{}'", b);
        println!("  Tokens: {:?}", tokens_b);

        let set_a: std::collections::HashSet<_> = tokens_a.iter().collect();
        let set_b: std::collections::HashSet<_> = tokens_b.iter().collect();
        let intersection: Vec<_> = set_a.intersection(&set_b).collect();
        let union: Vec<_> = set_a.union(&set_b).collect();
        let jaccard = intersection.len() as f64 / union.len() as f64;
        println!("  Intersection: {:?}", intersection);
        println!("  Jaccard: {:.3}", jaccard);
        println!();
    }
}
