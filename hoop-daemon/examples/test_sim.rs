use hoop_daemon::embedding::{cosine_similarity, Embedder, NgramEmbedder};

fn main() {
    let embedder = NgramEmbedder::new();

    let pairs = vec![
        (
            "Fix race condition in DB connection pool",
            "Fix database connection pool race condition",
        ),
        (
            "Implement user authentication with OAuth2",
            "Add OAuth2 user authentication",
        ),
        (
            "Add rate limiting to API endpoints",
            "Implement API endpoint rate limiting",
        ),
        (
            "Refactor database query builder",
            "Rewrite database query builder",
        ),
        (
            "Fix memory leak in worker process",
            "Repair worker process memory leak",
        ),
        (
            "Add pagination to list endpoints",
            "Implement pagination for list endpoints",
        ),
        (
            "Set up CI/CD pipeline for deploys",
            "Configure continuous deployment pipeline",
        ),
        (
            "Implement caching layer with Redis",
            "Add Redis caching for performance",
        ),
        (
            "Fix timezone handling in scheduler",
            "Repair scheduler timezone handling",
        ),
        (
            "Add WebSocket support for live updates",
            "Implement WebSocket for real-time updates",
        ),
        (
            "Fix auth race condition bug",
            "Fix authentication race condition bug",
        ),
        (
            "Setup config for production deploy",
            "Configure production deployment settings",
        ),
        (
            "Add user model CRUD operations",
            "Implement CRUD for user model",
        ),
        (
            "Refactor ORM mapping layer",
            "Restructure ORM layer mappings",
        ),
        (
            "Fix SSL certificate validation error",
            "Repair SSL certificate validation",
        ),
        (
            "Implement DNS resolution caching",
            "Add DNS caching for resolution",
        ),
        (
            "Add VPN tunnel support",
            "Implement VPN tunnel functionality",
        ),
        (
            "Fix async task queue deadlock",
            "Repair async queue deadlock issue",
        ),
        (
            "Implement RPC error handling",
            "Add error handling for RPC calls",
        ),
        (
            "Add HTML sanitizer for user input",
            "Implement HTML sanitization for input",
        ),
    ];

    let mut below_082 = 0;
    let mut below_075 = 0;
    for (a, b) in &pairs {
        let emb_a = embedder.embed(a);
        let emb_b = embedder.embed(b);
        let sim = cosine_similarity(&emb_a, &emb_b);
        if sim < 0.82 {
            below_082 += 1;
            if sim < 0.75 {
                below_075 += 1;
                println!("{:.3} - '{}' vs '{}'", sim, a, b);
            }
        }
    }
    println!("\nTotal below 0.82: {}/{}", below_082, pairs.len());
    println!("Total below 0.75: {}/{}", below_075, pairs.len());
}
