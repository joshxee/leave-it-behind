# ui

Score text in the top-left, in the bundled Super Indie font (or the default font when there is no `AssetServer`, as in headless tests).

- **Components:** `ScoreText`.
- **Systems:** `spawn_score_text` (Startup), `update_score_text` (Update/Present, runs when `Score` changes).
- **Functions:** `score_label`, `game_font`.
- **Tests:** covered visually by `e2e/specs/visual.spec.ts` and the interaction screenshots.
- **Scenarios:** `default`, `score_nine`.
