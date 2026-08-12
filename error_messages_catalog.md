# Error Messages Catalog - HOOP Tests

Generated: Wed Aug 12 09:57:30 AM EDT 2026

## Summary

- `.expect()` patterns: 373
- `.unwrap_err()` patterns: 10
- `anyhow!()` patterns: 1
- `anyhow::bail!()` patterns: 22
- `.context()` patterns: 2

## Error Type Patterns

### `.expect()` patterns

- **File:** `/home/coding/HOOP/tests/cli_test_helpers.rs:398`
  - **Message:** `.expect("projects.rs must exist"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/tests/cli_test_helpers.rs:412`
  - **Message:** `.expect("main.rs must exist"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:238`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:240`
  - **Message:** `.expect("Failed to create test client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:243`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:247`
  - **Message:** `.expect("Ready check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:256`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:258`
  - **Message:** `.expect("Failed to create test client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:265`
  - **Message:** `.expect("Failed to connect to WebSocket"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:271`
  - **Message:** `.expect("Timeout waiting for first message"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:272`
  - **Message:** `.expect("WebSocket stream ended"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:274`
  - **Message:** `.expect("Failed to receive first message"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:278`
  - **Message:** `.expect("Failed to parse init event"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:289`
  - **Message:** `.expect("subscriptions should be array"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:308`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:310`
  - **Message:** `.expect("Failed to create test client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:312`
  - **Message:** `.expect("Failed to collect snapshots"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:342`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:344`
  - **Message:** `.expect("Failed to create test client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:347`
  - **Message:** `.expect("Failed to collect WS snapshots"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:350`
  - **Message:** `.expect("Failed to fetch beads via REST"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:351`
  - **Message:** `.expect("Failed to fetch workers via REST"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:352`
  - **Message:** `.expect("Failed to fetch projects via REST"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:353`
  - **Message:** `.expect("Failed to fetch config via REST"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:390`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:392`
  - **Message:** `.expect("Failed to create test client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:395`
  - **Message:** `.expect("Failed to fetch beads"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:399`
  - **Message:** `.expect("Failed to fetch workers"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:403`
  - **Message:** `.expect("Failed to fetch projects"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:413`
  - **Message:** `.expect("Failed to fetch config status"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:417`
  - **Message:** `.expect("Failed to fetch capacity"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:426`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:428`
  - **Message:** `.expect("Failed to create test client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:430`
  - **Message:** `.expect("Failed to fetch metrics"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:461`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:468`
  - **Message:** `.expect("Failed to connect"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:475`
  - **Message:** `.expect("Timeout waiting for init"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:476`
  - **Message:** `.expect("Stream ended"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:477`
  - **Message:** `.expect("Failed to receive init"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:491`
  - **Message:** `.expect("Failed to send subscribe"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:500`
  - **Message:** `.expect("Failed to send unsubscribe"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:513`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:525`
  - **Message:** `.expect(&format!("Failed to connect (iteration {}))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:532`
  - **Message:** `.expect(&format!("Timeout (conn {}))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:533`
  - **Message:** `.expect("Stream ended"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:535`
  - **Message:** `.expect(&format!("No init (conn {}))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:539`
  - **Message:** `.expect("Failed to parse"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:552`
  - **Message:** `.expect("Task failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:561`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:570`
  - **Message:** `.expect("Failed to connect first time"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:577`
  - **Message:** `.expect("Timeout on first connection"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:578`
  - **Message:** `.expect("Stream ended"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:579`
  - **Message:** `.expect("No init on first connection"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:590`
  - **Message:** `.expect("Failed to reconnect"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:597`
  - **Message:** `.expect("Timeout on reconnection"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:598`
  - **Message:** `.expect("Stream ended"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:599`
  - **Message:** `.expect("No init on reconnection"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:609`
  - **Message:** `.expect("Timeout waiting for snapshots after reconnect"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:610`
  - **Message:** `.expect("Stream ended"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:611`
  - **Message:** `.expect("No snapshots after reconnect"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:630`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:632`
  - **Message:** `.expect("Failed to create test client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:635`
  - **Message:** `.expect("Failed to fetch beads"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:652`
  - **Message:** `.expect("Failed to fetch workers"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:665`
  - **Message:** `.expect("Failed to fetch projects"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:155`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:157`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:160`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:164`
  - **Message:** `.expect("Failed to spawn agent"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:172`
  - **Message:** `.expect("Failed to get agent status"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:176`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:187`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:189`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:192`
  - **Message:** `.expect("Failed to spawn agent"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:197`
  - **Message:** `.expect("Should have session_db_id"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:203`
  - **Message:** `.expect("Failed to get agent status"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:215`
  - **Message:** `.expect("Failed to switch adapter"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:220`
  - **Message:** `.expect("Should have new session_db_id"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:232`
  - **Message:** `.expect("Failed to list sessions"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:252`
  - **Message:** `.expect("Failed to get agent status"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:265`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:267`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:270`
  - **Message:** `.expect("Failed to spawn agent"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:275`
  - **Message:** `.expect("Should have session_db_id"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:281`
  - **Message:** `.expect("Failed to switch adapter"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:287`
  - **Message:** `.expect("Failed to list sessions"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:293`
  - **Message:** `.expect("Should find archived session"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:311`
  - **Message:** `.expect("Failed to query stitch from fleet.db"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:344`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:346`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:367`
  - **Message:** `.expect("Failed to insert reflection entry"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:370`
  - **Message:** `.expect("Failed to spawn agent"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:374`
  - **Message:** `.expect("Failed to switch adapter"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:378`
  - **Message:** `.expect("Failed to list reflection entries"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:389`
  - **Message:** `.expect("Entry should exist"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:402`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:404`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:407`
  - **Message:** `.expect("Failed to spawn agent"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:411`
  - **Message:** `.expect("Should have session_db_id"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:417`
  - **Message:** `.expect("Failed to switch adapter"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:423`
  - **Message:** `.expect("Failed to switch adapter back"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:427`
  - **Message:** `.expect("Should have second session_db_id"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:433`
  - **Message:** `.expect("Failed to list sessions"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:450`
  - **Message:** `.expect("Should find first archived session"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:454`
  - **Message:** `.expect("Should find second archived session"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:480`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:482`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:485`
  - **Message:** `.expect("Failed to spawn agent"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:506`
  - **Message:** `.expect("Failed to insert reflection entry"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:512`
  - **Message:** `.expect("Failed to switch adapter"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:518`
  - **Message:** `.expect("Failed to get agent status"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:524`
  - **Message:** `.expect("Failed to list reflection entries"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:539`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:541`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:544`
  - **Message:** `.expect("Failed to spawn agent"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:567`
  - **Message:** `.expect("Switch 1 should complete"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:570`
  - **Message:** `.expect("Switch 2 should complete"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:579`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:597`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:599`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:602`
  - **Message:** `.expect("Failed to spawn agent"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:607`
  - **Message:** `.expect("Should have session_db_id"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:613`
  - **Message:** `.expect("Failed to get agent status"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:639`
  - **Message:** `.expect("Failed to write updated config.yml"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:650`
  - **Message:** `.expect("Failed to get agent status after config reload"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:663`
  - **Message:** `.expect("Failed to list sessions"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:683`
  - **Message:** `.expect("Should find original archived session"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:700`
  - **Message:** `.expect("Failed to query stitch from fleet.db"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:722`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:805`
  - **Message:** `.expect("Failed to start mock Anthropic server"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:812`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:814`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:817`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:837`
  - **Message:** `.expect("Failed to write config with mock server URL"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:853`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:865`
  - **Message:** `.expect("Ready endpoint request failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:882`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:892`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:910`
  - **Message:** `.expect("Failed to start mock Anthropic server"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:917`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:919`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:922`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:937`
  - **Message:** `.expect("Failed to write config"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:943`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:950`
  - **Message:** `.expect("Adapter switch should succeed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:958`
  - **Message:** `.expect("Failed to get agent status"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:963`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/stdout_generation_test.rs:150`
  - **Message:** `.expect("Failed to execute subprocess"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/stdout_generation_test.rs:183`
  - **Message:** `.expect("Failed to execute test binary"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/stdout_generation_test.rs:266`
  - **Message:** `.expect("Failed to execute multi-line subprocess"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:258`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:260`
  - **Message:** `.expect("Failed to create test client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:263`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:267`
  - **Message:** `.expect("Ready check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:276`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:278`
  - **Message:** `.expect("Failed to create test client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:285`
  - **Message:** `.expect("Failed to connect to WebSocket"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:291`
  - **Message:** `.expect("Timeout waiting for first message"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:292`
  - **Message:** `.expect("WebSocket stream ended"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:294`
  - **Message:** `.expect("Failed to receive first message"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:298`
  - **Message:** `.expect("Failed to parse init event"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:309`
  - **Message:** `.expect("subscriptions should be array"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:317`
  - **Message:** `.expect("subscriptions should be array"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:336`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:338`
  - **Message:** `.expect("Failed to create test client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:340`
  - **Message:** `.expect("Failed to collect snapshots"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:370`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:372`
  - **Message:** `.expect("Failed to create test client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:375`
  - **Message:** `.expect("Failed to fetch beads"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:380`
  - **Message:** `.expect("Failed to fetch workers"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:385`
  - **Message:** `.expect("Failed to fetch conversations"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:390`
  - **Message:** `.expect("Failed to fetch projects"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:395`
  - **Message:** `.expect("Failed to fetch config status"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:399`
  - **Message:** `.expect("Failed to fetch capacity"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:409`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:411`
  - **Message:** `.expect("Failed to create test client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:413`
  - **Message:** `.expect("Failed to fetch metrics"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:444`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:451`
  - **Message:** `.expect("Failed to connect"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:458`
  - **Message:** `.expect("Timeout waiting for init"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:459`
  - **Message:** `.expect("Stream ended"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:460`
  - **Message:** `.expect("Failed to receive init"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:474`
  - **Message:** `.expect("Failed to send subscribe"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:483`
  - **Message:** `.expect("Failed to send unsubscribe"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:496`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:508`
  - **Message:** `.expect(&format!("Failed to connect (iteration {}))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:515`
  - **Message:** `.expect(&format!("Timeout (conn {}))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:516`
  - **Message:** `.expect("Stream ended"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:518`
  - **Message:** `.expect(&format!("No init (conn {}))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:522`
  - **Message:** `.expect("Failed to parse"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:535`
  - **Message:** `.expect("Task failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:544`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:553`
  - **Message:** `.expect("Failed to connect first time"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:560`
  - **Message:** `.expect("Timeout on first connection"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:561`
  - **Message:** `.expect("Stream ended"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:562`
  - **Message:** `.expect("No init on first connection"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:573`
  - **Message:** `.expect("Failed to reconnect"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:580`
  - **Message:** `.expect("Timeout on reconnection"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:581`
  - **Message:** `.expect("Stream ended"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:582`
  - **Message:** `.expect("No init on reconnection"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:592`
  - **Message:** `.expect("Timeout waiting for snapshots after reconnect"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:593`
  - **Message:** `.expect("Stream ended"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:594`
  - **Message:** `.expect("No snapshots after reconnect"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:613`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:615`
  - **Message:** `.expect("Failed to create test client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:618`
  - **Message:** `.expect("Failed to fetch beads"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:635`
  - **Message:** `.expect("Failed to fetch workers"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:645`
  - **Message:** `.expect("Failed to fetch projects"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:72`
  - **Message:** `.expect("Failed to spawn daemon with load test data"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:81`
  - **Message:** `.expect("Health check request failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:100`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:167`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:219`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:286`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:345`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:350`
  - **Message:** `.expect("Load test failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:355`
  - **Message:** `.expect("Performance budget violations detected"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:380`
  - **Message:** `.expect("Failed to populate testrepo with load test data"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:396`
  - **Message:** `.expect("Failed to create project directory"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:410`
  - **Message:** `.expect("Failed to serialize projects.yaml"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:412`
  - **Message:** `.expect("Failed to write projects.yaml"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:467`
  - **Message:** `.expect("Failed to spawn daemon with load test data"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:472`
  - **Message:** `.expect("Failed to write daemon URL to file"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:480`
  - **Message:** `.expect("Load test failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:490`
  - **Message:** `.expect("Performance budget violations detected - blocking merge per hoop-ttb.7.11"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:528`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test_integration.rs:557`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test.rs:209`
  - **Message:** `.expect("Failed to spawn test daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test.rs:214`
  - **Message:** `.expect("Load test should complete"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test.rs:262`
  - **Message:** `.expect("Failed to spawn test daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test.rs:271`
  - **Message:** `.expect("Load test timed out after 10 minutes"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test.rs:272`
  - **Message:** `.expect("Load test should complete"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test.rs:282`
  - **Message:** `.expect("Performance budgets must be satisfied"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test.rs:329`
  - **Message:** `.expect("Failed to spawn test daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test.rs:337`
  - **Message:** `.expect("Medium-scale load test timed out"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test.rs:338`
  - **Message:** `.expect("Load test should complete"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/load_test.rs:345`
  - **Message:** `.expect("Medium-scale load test should pass performance budgets"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/examples/populate-testrepo.rs:37`
  - **Message:** `.expect("workspace root is parent of hoop-daemon/"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:152`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:154`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:157`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:161`
  - **Message:** `.expect("Failed to spawn agent"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:169`
  - **Message:** `.expect("Failed to get agent status"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:173`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:184`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:186`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:189`
  - **Message:** `.expect("Failed to spawn agent"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:194`
  - **Message:** `.expect("Should have session_db_id"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:200`
  - **Message:** `.expect("Failed to get agent status"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:212`
  - **Message:** `.expect("Failed to switch adapter"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:217`
  - **Message:** `.expect("Should have new session_db_id"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:229`
  - **Message:** `.expect("Failed to list sessions"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:249`
  - **Message:** `.expect("Failed to get agent status"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:262`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:264`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:267`
  - **Message:** `.expect("Failed to spawn agent"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:272`
  - **Message:** `.expect("Should have session_db_id"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:278`
  - **Message:** `.expect("Failed to switch adapter"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:284`
  - **Message:** `.expect("Failed to list sessions"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:290`
  - **Message:** `.expect("Should find archived session"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:308`
  - **Message:** `.expect("Failed to query stitch from fleet.db"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:341`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:343`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:364`
  - **Message:** `.expect("Failed to insert reflection entry"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:367`
  - **Message:** `.expect("Failed to spawn agent"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:371`
  - **Message:** `.expect("Failed to switch adapter"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:375`
  - **Message:** `.expect("Failed to list reflection entries"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:386`
  - **Message:** `.expect("Entry should exist"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:399`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:401`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:404`
  - **Message:** `.expect("Failed to spawn agent"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:408`
  - **Message:** `.expect("Should have session_db_id"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:414`
  - **Message:** `.expect("Failed to switch adapter"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:420`
  - **Message:** `.expect("Failed to switch adapter back"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:424`
  - **Message:** `.expect("Should have second session_db_id"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:430`
  - **Message:** `.expect("Failed to list sessions"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:447`
  - **Message:** `.expect("Should find first archived session"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:451`
  - **Message:** `.expect("Should find second archived session"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:477`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:479`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:482`
  - **Message:** `.expect("Failed to spawn agent"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:503`
  - **Message:** `.expect("Failed to insert reflection entry"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:509`
  - **Message:** `.expect("Failed to switch adapter"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:515`
  - **Message:** `.expect("Failed to get agent status"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:521`
  - **Message:** `.expect("Failed to list reflection entries"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:536`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:538`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:541`
  - **Message:** `.expect("Failed to spawn agent"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:568`
  - **Message:** `.expect("Switch 1 should complete"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:571`
  - **Message:** `.expect("Switch 2 should complete"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:580`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:597`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:599`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:602`
  - **Message:** `.expect("Failed to spawn agent"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:607`
  - **Message:** `.expect("Should have session_db_id"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:613`
  - **Message:** `.expect("Failed to get agent status"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:639`
  - **Message:** `.expect("Failed to write updated config.yml"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:650`
  - **Message:** `.expect("Failed to get agent status after config reload"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:663`
  - **Message:** `.expect("Failed to list sessions"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:683`
  - **Message:** `.expect("Should find original archived session"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:700`
  - **Message:** `.expect("Failed to query stitch from fleet.db"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:722`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:808`
  - **Message:** `.expect("Failed to start mock Anthropic server"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:815`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:817`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:820`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:840`
  - **Message:** `.expect("Failed to write config with mock server URL"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:856`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:868`
  - **Message:** `.expect("Ready endpoint request failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:885`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:895`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:913`
  - **Message:** `.expect("Failed to start mock Anthropic server"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:920`
  - **Message:** `.expect("Failed to spawn daemon"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:922`
  - **Message:** `.expect("Failed to create client"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:925`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:940`
  - **Message:** `.expect("Failed to write config"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:946`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:953`
  - **Message:** `.expect("Adapter switch should succeed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:961`
  - **Message:** `.expect("Failed to get agent status"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:966`
  - **Message:** `.expect("Health check failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/clap_test_utils.rs:681`
  - **Message:** `.expect("Should parse with flag before command"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/clap_test_utils.rs:703`
  - **Message:** `.expect("Should parse with flag after command"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/clap_test_utils.rs:725`
  - **Message:** `.expect("Should parse with -y flag"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/clap_test_utils.rs:771`
  - **Message:** `.expect("Should parse without flag"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/clap_test_utils.rs:803`
  - **Message:** `.expect("Should parse with flag before command"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/clap_test_utils.rs:811`
  - **Message:** `.expect("Should parse with flag after command"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/clap_test_utils.rs:819`
  - **Message:** `.expect("Should parse with -y flag"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/clap_test_utils.rs:840`
  - **Message:** `.expect("Should parse without flag"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:407`
  - **Message:** `.expect("Failed to create .beads/ directory"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:414`
  - **Message:** `.expect("Failed to create .hoop/ directory"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:423`
  - **Message:** `.expect("Failed to write projects.yaml"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:586`
  - **Message:** `.expect("Failed to parse with flag before command"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:595`
  - **Message:** `.expect("Failed to parse with flag after command"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:757`
  - **Message:** `.expect("Failed to parse with flag before subcommand"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:764`
  - **Message:** `.expect("Failed to parse with flag after subcommand"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:782`
  - **Message:** `.expect("Failed to parse with -y flag"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:788`
  - **Message:** `.expect("Failed to parse without flag"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:800`
  - **Message:** `.expect("Failed to parse with flag before subcommand"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:805`
  - **Message:** `.expect("Failed to parse with flag after subcommand"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:890`
  - **Message:** `.expect("Failed to create temp dir"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:917`
  - **Message:** `.expect("Failed to create temp dir"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:926`
  - **Message:** `.expect("Failed to parse remove with flag before"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:933`
  - **Message:** `.expect("Failed to parse remove with flag after"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:1139`
  - **Message:** `.expect("Failed to create temp dir"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils.rs:1148`
  - **Message:** `.expect("Failed to create temp dir"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:229`
  - **Message:** `.expect("Failed to read main.rs"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:753`
  - **Message:** `.expect("Failed to read mycommand.rs"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:770`
  - **Message:** `.expect("Failed to read main.rs"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:804`
  - **Message:** `.expect("Failed to read projects.rs"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:835`
  - **Message:** `.expect("Failed to read init.rs"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:876`
  - **Message:** `.expect("Failed to read main.rs"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:891`
  - **Message:** `.expect("Failed to read projects.rs"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2174`
  - **Message:** `.expect("Flag before subcommand assertion failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2195`
  - **Message:** `.expect("Flag after subcommand assertion failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2237`
  - **Message:** `.expect("Default flag assertion failed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2830`
  - **Message:** `.expect("Should parse flag before subcommand"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2838`
  - **Message:** `.expect("Should parse flag after subcommand"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2846`
  - **Message:** `.expect("Should parse short flag"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2855`
  - **Message:** `.expect("Should parse nested command"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2866`
  - **Message:** `.expect("Should parse nested command with flag"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2882`
  - **Message:** `.expect("Should parse command with multiple flags"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2890`
  - **Message:** `.expect("Should parse command without flag"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2911`
  - **Message:** `.expect("Should parse successfully"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2923`
  - **Message:** `.expect("Should parse flag-only args"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils_examples.rs:246`
  - **Message:** `.expect("Failed to create temp dir"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils_examples.rs:259`
  - **Message:** `.expect("Failed to create temp dir"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils_examples.rs:270`
  - **Message:** `.expect("Failed to read registry file"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils_examples.rs:279`
  - **Message:** `.expect("Failed to create temp dir"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils_examples.rs:313`
  - **Message:** `.expect("Should parse remove command successfully"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils_examples.rs:405`
  - **Message:** `.expect("Should parse successfully"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils_examples.rs:433`
  - **Message:** `.expect("Failed to create temp dir"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils_examples.rs:440`
  - **Message:** `.expect("Parse with flag before should succeed"))`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils_examples.rs:446`
  - **Message:** `.expect("Parse with flag after should succeed"))`
  - **Type:** Error type

### `.unwrap_err()` patterns

- **File:** `/home/coding/HOOP/hoop-daemon/tests/mutation_handler_test.rs:163`
  - **Message:** `.unwrap_err()`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/mutation_handler_test.rs:205`
  - **Message:** `.unwrap_err()`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-daemon/tests/mutation_handler_test.rs:238`
  - **Message:** `.unwrap_err()`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2606`
  - **Message:** `.unwrap_err()`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2613`
  - **Message:** `.unwrap_err()`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2620`
  - **Message:** `.unwrap_err()`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2651`
  - **Message:** `.unwrap_err()`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_helpers.rs:2665`
  - **Message:** `.unwrap_err()`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils_examples.rs:397`
  - **Message:** `.unwrap_err()`
  - **Type:** Error type

- **File:** `/home/coding/HOOP/hoop-cli/tests/cli_test_utils_examples.rs:423`
  - **Message:** `.unwrap_err()`
  - **Type:** Error type

## anyhow Error Patterns

### `anyhow!()` patterns

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:322`
  - **Message:** `anyhow!("WebSocket error: {}", e))`
  - **Type:** anyhow error

### `anyhow::bail!()` patterns

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_integration.rs:67`
  - **Message:** `anyhow::bail!("Daemon did not become ready"))`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/tests/adapter_failover_test.rs:49`
  - **Message:** `anyhow::bail!("Daemon did not become ready"))`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/tests/testrepo_harness_integration.rs:57`
  - **Message:** `anyhow::bail!("Daemon did not become ready"))`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/tests_phase5/adapter_failover_test.rs:46`
  - **Message:** `anyhow::bail!("Daemon did not become ready"))`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:69`
  - **Message:** `anyhow::bail!("Daemon did not become ready within {:?}", timeout))`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:99`
  - **Message:** `anyhow::bail!("GET /api/beads failed: {}", resp.status())`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:115`
  - **Message:** `anyhow::bail!("GET /api/beads/{} failed: {}", bead_id, resp.status())`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:178`
  - **Message:** `anyhow::bail!("GET /api/capacity failed: {}", resp.status())`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:193`
  - **Message:** `anyhow::bail!("GET /metrics failed: {}", resp.status())`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:208`
  - **Message:** `anyhow::bail!("GET /api/workers/timeline failed: {}", resp.status())`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:234`
  - **Message:** `anyhow::bail!("Health check failed: {}", resp.status())`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:243`
  - **Message:** `anyhow::bail!("Readiness check failed: {}", resp.status())`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:252`
  - **Message:** `anyhow::bail!("Bead ID mismatch: expected {}, got {}", bead_id, bead["id"]))`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:262`
  - **Message:** `anyhow::bail!("No bead with title '{}' found", title))`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:286`
  - **Message:** `anyhow::bail!("Capacity response is not an object"))`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:352`
  - **Message:** `anyhow::bail!("WebSocket connection closed"))`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:355`
  - **Message:** `anyhow::bail!("WebSocket connection terminated"))`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:361`
  - **Message:** `anyhow::bail!("Timeout waiting for bead event"))`
  - **Type:** anyhow bail

- **File:** `/home/coding/HOOP/hoop-daemon/src/load_test.rs:369`
  - **Message:** `anyhow::bail!("Performance budget violations:\n{}", failures.join("\n"))`
  - **Type:** anyhow bail

### `.context()` patterns

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:314`
  - **Message:** `.context("Failed to send WebSocket message"))`
  - **Type:** anyhow context

- **File:** `/home/coding/HOOP/hoop-daemon/src/integration_test_client.rs:384`
  - **Message:** `.context("Failed to send close message"))`
  - **Type:** anyhow context

