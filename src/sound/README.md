# Sound

The ship is mostly silent. A sliding door quietly plays the sci-fi door clip when it begins opening and the supplied clang once when it finishes closing. The locked hatch never moves or sounds. Ambient sound waits 45 seconds after each flight begins; thereafter an eight-second excerpt of the lab ambience fades in and out at very low volume at random intervals of 95–145 seconds. There is no continuous loop or music.

Each valid wrench turn plays a short part of the supplied wrench recording; canceling the turn stops it. Tape feeds while tape is actually being laid, and the engine loop plays only while the helm is engaged and the player is steering with WASD. An open hull breach is audible only in its room, fades with distance and repair progress, and stops when sealed. Active sounds fade quickly as actions stop. All sound entities pause with gameplay, resume with it, and stop on the outcome screens.

`SoundPlugin` loads the seven listed runtime clips from `assets/RUNTIME_ASSETS`. `AmbienceClock` resets each flight and advances only while running. Headless test apps omit the `AssetServer`, so they exercise sound decisions without creating audio players.

The original source files remain under `assets/audio/`. The sci-fi door and lab ambience are CC0 according to the Freesound collection license text. Keep the clank's supplied license PDF with its source file. `assets/audio/gameplay/CREDITS.txt` records sources and known license terms for the four newer sounds, including the wrench clip's noncommercial restriction. Verify the tape and breach licenses before release.
