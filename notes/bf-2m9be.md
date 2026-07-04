# WebSocket Init Event Subscription Implementation - Findings

## Task
Review the WebSocket init event implementation to understand how subscriptions are currently being sent.

## Current Subscription Array Construction Logic

### Location
`hoop-daemon/src/ws.rs`, lines 1659-1678

### Implementation Details

**1. Initial subscription set creation (lines 1659-1663):**
```rust
let subscriptions: Arc<tokio::sync::RwLock<HashSet<String>>> =
    Arc::new(tokio::sync::RwLock::new(["global".to_string()].into()));
```

The subscriptions are initialized with **only the "global" topic** in a `HashSet<String>` wrapped in an `Arc<RwLock<>>` for thread-safe access.

**2. Init event construction and sending (lines 1672-1678):**
```rust
// 1. init — client must wipe its state and adopt this subscription list.
let init_subs: Vec<String> = subscriptions.read().await.iter().cloned().collect();
if let Ok(json) = serde_json::to_string(&WsEvent::init(init_subs)) {
    if sender.send(Message::Text(json)).await.is_err() {
        return;
    }
}
```

The implementation:
- Reads the current subscriptions (which is just `["global"]`)
- Clones them into a `Vec<String>`
- Creates an `WsEvent::init()` with that vector
- Serializes and sends the event

**3. WsEvent::init() constructor (lines 1413-1444):**
```rust
pub fn init(subs: Vec<String>) -> Self {
    Self {
        event_type: "init".to_string(),
        // ... all other fields set to None ...
        subscriptions: Some(subs),
    }
}
```

**4. WsEvent structure (lines 752-808):**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WsEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    // ... many optional fields ...
    /// Present only on `init` events; the server-authoritative subscription list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscriptions: Option<Vec<String>>,
}
```

## Global Subscription Inclusion

✅ **YES, the global subscription IS included.**

The implementation explicitly initializes subscriptions with `["global"]` (line 1663), which means the init event will always contain:
```json
{
  "type": "init",
  "subscriptions": ["global"]
}
```

## Test Expectations

### Unit Tests (ws.rs lines 2365-2383)

1. **`init_event_carries_subscriptions`**: Tests that `WsEvent::init()` carries the subscriptions it's given
2. **`init_event_serializes_subscriptions_field`**: Tests that the init event serializes with the subscriptions field containing "global"
3. **`non_init_events_omit_subscriptions_field`**: Tests that other events don't include subscriptions field

### Integration Test (integration_harness.rs lines 809-813)

```rust
assert_eq!(event["type"], "init", "First message should be init event");
assert!(
    event["subscriptions"].is_array(),
    "init should contain subscriptions"
);
```

The integration test only validates:
- Event type is "init"
- Subscriptions field exists and is an array

It does NOT validate the specific content of the subscriptions array.

## Key Architecture Points

### Server-First Epoch Principle (§3)
- The init event tells the client the authoritative subscription state
- Clients must wipe their local state and adopt this list on receipt
- Server is the source of truth on reconnect

### Default Subscription Policy
- **Global subscription is server-pinned**: Clients automatically start with "global" subscription
- **Cannot unsubscribe from global**: The inbound message handler (lines 2139-2143) explicitly prevents removal of "global":
  ```rust
  Ok(ClientMessage::Unsubscribe { topic }) => {
      // global is server-pinned and cannot be removed
      if topic != "global" {
          subs_for_recv.write().await.remove(&topic);
      }
  }
  ```

### Topic Routing (§5.2)
- Events can be routed to specific topics:
  - `"global"` — cross-project events (no topic specified in WsOutMsg)
  - `"project:<name>"` — project-scoped events
- `should_deliver()` function (lines 86-91) determines delivery based on topic matching

## Summary

✅ **Implementation is correct and complete:**
1. Init event is properly constructed with subscriptions array
2. Global subscription (`"global"`) is included by default
3. Server enforces "global" as a pinned, non-removable subscription
4. Test expectations align with implementation
5. Architecture follows "server is the epoch" principle

**No discrepancies found between implementation and test expectations.**
