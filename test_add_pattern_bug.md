# Bug Analysis: add_pattern() in hoop-daemon/src/risk_patterns.rs

## Status: Cannot Verify Due to Compilation Errors

The codebase currently has **95 compilation errors** that prevent running the test. This is a known state per AGENTS.md: "The Rust crate does NOT currently compile."

## The Implementation

```rust
pub fn add_pattern(&mut self, pattern: RiskPattern) {
    let idx = self.patterns.len();  // Gets current length BEFORE push
    for keyword in &pattern.keywords {
        self.keyword_index
            .entry(keyword.to_lowercase())
            .or_default()
            .push(idx);
    }
    for label_keyword in &pattern.label_keywords {
        self.label_keyword_index
            .entry(label_keyword.to_lowercase())
            .or_default()
            .push(idx);
    }
    self.patterns.push(pattern);  // Pattern pushed AFTER index used
}
```

## Logic Analysis

The implementation appears **logically correct**. Here's the trace:

### Adding First Pattern
```
Library: patterns = []
1. idx = patterns.len() = 0
2. keyword_index["test"] = [0]
3. patterns.push() → patterns[0] = test_pattern ✓
```

### Adding Second Pattern  
```
Library: patterns = [pattern1]
1. idx = patterns.len() = 1
2. keyword_index["other"] = [1]
3. patterns.push() → patterns[1] = pattern2 ✓
```

### Matching
```
match_draft("Test this", None, []):
- text = "test this"
- keyword_index["test"] = [0]
- RiskMatchBuilder::new(&patterns[0]) → accesses patterns[0] ✓
```

## What the Test Expects

```rust
#[test]
fn test_add_pattern() {
    let mut lib = FixLineageLibrary::new();
    lib.add_pattern(RiskPattern {
        id: "test_pattern".to_string(),
        keywords: vec!["test".to_string()],
        // ...
    });

    let matches = lib.match_draft("Test this", None, &[]);
    assert_eq!(matches.len(), 1);           // Exactly one match
    assert_eq!(matches[0].pattern.id, "test_pattern");  // Correct pattern ID
}
```

Expected behavior:
1. Add pattern with keyword "test"
2. Match draft containing "Test" (case-insensitive)
3. Return single match with id "test_pattern"
4. Match confidence should be 0.3 (one keyword match)

## Implementation vs Test Expectations

✓ **Correctly** adds pattern to patterns Vec
✓ **Correctly** indexes keywords by position
✓ **Correctly** positions pattern at expected index
✓ **Correctly** enables match_draft to find the pattern

## Conclusion

**The implementation appears logically sound**. The bug cannot be verified without fixing the 95 compilation errors in the codebase.

**If the test were to run**, it would likely:
1. **PASS** if the logic is correct
2. **FAIL** if there's a subtle edge case not visible from code inspection

**Recommendation**: Fix compilation errors first, then run test to verify actual behavior.
