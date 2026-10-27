# Invisible Deck Helper

A practice aid for the **Invisible Deck** card trick, written in Rust, packaged as
a Flatpak for Kubuntu 26.04.

Shows one card at a time, so you can drill the effect: a spectator names a card,
you spread the deck, and the only face-up card is the one they named. A toggle —
**off by default** — reveals the invisible partner card glued to its back, so you
can check yourself before flipping it over.

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
| Reveal the pair | Tick **Show Invisible Deck pair** (starts off) |

Command line, for checking things without a window:

```bash
invisible-deck-helper --print            # one card + its partner
invisible-deck-helper --print --seed 42  # repeatable
invisible-deck-helper --pairs            # all 26 pairings
invisible-deck-helper --version
```

## Building natively

Needs Rust 1.88+ (the egui stack requires it; Debian 13's rustc 1.85 is too old)
and the usual OpenGL/X11 dev bits.

```bash
cargo test                     # pairing + RNG tests
cargo build --release
./target/release/invisible-deck-helper
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

The script vendors the Rust crates into `cargo-sources.json` on first run (using
the official `flatpak-cargo-generator.py`), installs `org.freedesktop.Platform`,
`org.freedesktop.Sdk` and the `rust-stable` SDK extension for the **26.08**
branch, then builds offline.

Roughly 2 GB of runtimes are downloaded on first build.

## Layout

```
Cargo.toml                          crate manifest
src/cards.rs                        deck + Invisible Deck pairing (with tests)
src/rng.rs                          /dev/urandom live source, SplitMix64 when seeded
src/main.rs                         egui UI and the headless modes
data/*.desktop, *.metainfo.xml      app metadata for the desktop and Flathub tooling
data/icons/*.svg                    application icon
org.cydermaster.InvisibleDeckHelper.yml   Flatpak manifest
build-flatpak.sh                    one-command build + bundle
cargo-sources.json                  vendored crate sources (generated)
```
