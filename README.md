# Invisible Deck Helper

A practice aid for the **Invisible Deck** card trick, written in Rust, packaged as
a Flatpak for Kubuntu 26.04.

Shows one card at a time and **speaks it aloud**: a spectator names a card, you
spread the deck, and the only face-up card is the one they named. A toggle — **off
by default** — reveals the invisible partner card glued to its back, so you can
check yourself before flipping it over.

## Speech

Text to speech is [Piper](https://github.com/OHF-Voice/piper1-gpl) running
locally, with the **English (Great Britain) — Alan, medium quality** voice
(`en_GB-alan-medium`, 22 050 Hz). Nothing is sent anywhere.

- Pair toggle **off** (default): says just the card — *"Queen of Hearts"*
- Pair toggle **on**: says *"Queen of Hearts pairs with Ace of Spades"*

It speaks when you deal, when you flip the toggle, and when you press **Say
again**. Synthesis runs on a background thread so the window never freezes.
Playback uses `libpulse-simple` from the runtime (dlopen'd, so nothing is linked
at build time), falling back to `paplay` / `pw-play` / `aplay` / `ffplay` / `play`
if that is unavailable.

## The pairing scheme

Taken from the published Royal Magic / FUN Inc. Invisible Deck pairing guide:

- **Hearts pair with Spades; Diamonds pair with Clubs** — every pair is one red
  and one black card.
- **Each pair totals 13** — Ace (1) with Queen (12), 2 with Jack, 3 with 10,
  4 with 9, 5 with 8, 6 with 7. Both suit orders are used, so all 52 cards are
  covered.
- **Kings pair with Kings.**

In code that collapses to one rule: the partner of a card has rank `13 - rank`
(Kings keeping rank 13) and the opposite-colour partner suit. It is an
involution, so `card.partner().partner() == card` — asserted in the tests.

Run `invisible-deck-helper --pairs` to print all 26 pairings.

## Using it

| Action | How |
|---|---|
| Deal the next card | **Deal new card** button, or **Space** / **Enter** |
| Reveal + announce the pair | Tick **Show Invisible Deck pair** (starts off) |
| Repeat the speech | **Say again** |

Command line, for checking things without a window:

```bash
invisible-deck-helper --print              # card, partner and the phrase
invisible-deck-helper --print --pair       # phrase as if the toggle were on
invisible-deck-helper --print --seed 42    # repeatable
invisible-deck-helper --pairs              # all 26 pairings
invisible-deck-helper --speech-info        # which engine + voice is in use
invisible-deck-helper --say "Ten of Spades pairs with Three of Hearts"
invisible-deck-helper --version
```

## Building natively

Needs Rust 1.88+ (the egui stack requires it; Debian 13's rustc 1.85 is too old)
and the usual OpenGL/X11 dev bits. Speech additionally needs `piper` and the voice
model on the machine; point the app at them with `IDH_PIPER` and `IDH_DATA_DIR`.

```bash
cargo test                     # pairing + RNG tests
cargo build --release
./target/release/invisible-deck-helper --speech-info
```

## Building the Flatpak

```bash
./build-flatpak.sh            # builds, installs, and emits the .flatpak bundle
./build-flatpak.sh --bundle   # build + bundle only, no local install
```

Then:

```bash
flatpak run org.cydermaster.InvisibleDeckHelper
```

The Flatpak is self-contained for speech: it bundles the `piper-tts` wheel with
its whole dependency set (onnxruntime, numpy, protobuf, flatbuffers, packaging,
pathvalidate — all pinned to cp314 wheels, matching the runtime's Python 3.14),
plus the Alan voice model. `org.gnome.Platform//51` supplies Python and
`libpulse-simple`; the Rust toolchain comes from
`org.freedesktop.Sdk.Extension.rust-stable//26.08` (the branch the GNOME 51 SDK
declares).

Roughly 2.5 GB of runtimes are downloaded on first build. Afterwards the bundle
itself is small — it contains the app, the voice and the wheels, not the runtime.

Because the manifest fetches every wheel and the voice by **URL + sha256**, a
first build needs network access; the build itself then runs offline.

## Layout

```
Cargo.toml                          crate manifest
src/cards.rs                        deck + Invisible Deck pairing (with tests)
src/rng.rs                          /dev/urandom live source, SplitMix64 when seeded
src/speech.rs                       Piper invocation + WAV playback
src/main.rs                         egui UI and the headless modes
data/*.desktop, *.metainfo.xml      app metadata for the desktop and Flathub tooling
data/icons/*.svg                    application icon
org.cydermaster.InvisibleDeckHelper.yml   Flatpak manifest
build-flatpak.sh                    one-command build + bundle
cargo-sources.json                  vendored crate sources (generated)
```
