use hoop_daemon::embedding::{cosine_similarity, jaccard_similarity, Embedder, NgramEmbedder};

fn main() {
    let embedder = NgramEmbedder::new();

    let existing = "Implement OAuth2 authentication flow";
    let draft_title = "Implement OAuth2 auth flow";
    let draft_desc = "Set up OAuth2 provider for user login";

    let tokens_existing = embedder.canonical_tokens(existing);
    let tokens_title = embedder.canonical_tokens(draft_title);
    let tokens_desc = embedder.canonical_tokens(draft_desc);

    println!("Existing: '{}'", existing);
    println!("  Tokens: {:?}", tokens_existing);
    println!();
    println!("Draft title: '{}'", draft_title);
    println!("  Tokens: {:?}", tokens_title);
    println!();
    println!("Draft desc: '{}'", draft_desc);
    println!("  Tokens: {:?}", tokens_desc);
    println!();

    // Embed and check similarity
    let emb_existing = embedder.embed(existing);
    let emb_draft = embedder.embed(&format!("{} {}", draft_title, draft_desc));
    let cosine = cosine_similarity(&emb_existing, &emb_draft);
    let jaccard = jaccard_similarity(&tokens_existing, &tokens_title);

    println!("Cosine sim: {:.3}", cosine);
    println!("Jaccard sim: {:.3}", jaccard);

    // Combined with boost
    let base = cosine.max(jaccard);
    let boost = if cosine > 0.65 && jaccard > 0.65 {
        0.05 * cosine.min(jaccard)
    } else {
        0.0
    };
    println!(
        "Combined: {:.3} (base: {:.3}, boost: {:.3})",
        base + boost,
        base,
        boost
    );
}
