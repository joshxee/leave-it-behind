# scenarios

Fixtures that put the world into an exact situation. Shared by native tests (`test_app_with`), the e2e web build (`?scenario=<name>`), and `e2e/tools/capture.mjs --scenario`.

- **Types:** `Scenario` (`ALL`, `name`, `from_name`, `apply`), `ActiveScenario`.
- **Scenarios:** `default`, `score_nine`.
- **Tests:** `tests/integration/scenarios.rs` (every scenario boots).
- **Adding one:** see the template in `TESTING.md`.
