# audio

Plays `assets/audio/clank-sound/Clank.wav` on every `ScoreChanged`. Browsers block audio until the user interacts with the page (`wasm/restart-audio-context.js` resumes it).

- **Resources:** `Sfx` (private).
- **Systems:** `load_sfx` (Startup), `play_clank` (Update/Present).
- Not added without an `AssetServer`, so headless tests skip it.
- **Tests:** none (presentation only).
- **Scenarios:** none.
