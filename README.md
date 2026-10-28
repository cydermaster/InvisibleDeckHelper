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

**Silencing it.** Tick **Mute speech** in the window for a silent session — the
screen still shows `Muted — would say: "…"` so nothing is lost, and unticking it
restores the voice. To start silent, set `IDH_MUTE=1` (handy for scripting or for
practising somewhere you cannot make noise).

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
| Silence the voice | Tick **Mute speech** (starts off) |

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

## Windows 11

The same source builds for Windows. The window, the pairing engine and the
spoken phrases are shared; only three pieces differ — the OS random source
(`RtlGenRandom` instead of `/dev/urandom`), audio playback (`PlaySound` from
winmm instead of PulseAudio), and how Piper is found (an embedded
`python\python.exe` beside the exe instead of `/app/bin/piper`).

It ships as a portable folder — no installer, nothing installed on the machine:

```
InvisibleDeckHelper-<version>-windows-x86_64\
    invisible-deck-helper.exe
    python\          embedded CPython 3.11 carrying piper-tts and its dependencies
    voices\          en_GB-alan-medium.onnx
    README.txt
```

**Keep the folders beside the exe.** The exe on its own runs and deals cards, but
it cannot speak: everything it needs for speech lives in `python\` and `voices\`.

Build it either way:

```powershell
# on Windows, with MSVC
.\windows\build-portable-zip.ps1
```

```bash
# on Linux, cross-compiling with mingw-w64 (needs gcc-mingw-w64-x86-64)
./windows/build-portable-zip.sh
```

### Installer

For people who would rather have it in the Start Menu, `windows/installer.iss`
builds an Inno Setup installer from the same bundle — identical files, packaged
with an uninstaller:

```powershell
winget install --id JRSoftware.InnoSetup --exact   # once
.\windows\build-portable-zip.ps1                    # produces the bundle
.\windows\build-installer.ps1                       # wraps the bundle in a setup.exe
```

It installs **per-user** by default (`%LOCALAPPDATA%\Programs\Invisible Deck Helper`)
so there is no UAC prompt, and offers an all-users install if launched with
`/ALLUSERS`. It adds a Start Menu entry, an optional desktop shortcut, and
registers in Add or Remove Programs. Because the payload is ~240 MB compressed
with LZMA2, the setup file ends up roughly the size of the ZIP.

The installer is unsigned for now, so SmartScreen will warn on first run — see
the note below.

The voice model is fetched from Hugging Face and checked against a pinned
SHA-256 rather than committed — 63 MB of binary does not belong in git. Both
scripts accept `NUMPY_PIN='numpy==1.26.4'` as an escape hatch if a future numpy
release fails to load on a target machine.

The exe is **not code-signed**, so the first run shows Windows SmartScreen's
"Windows protected your PC" box: *More info* → *Run anyway*. Signing is the only
way to remove that, and it needs a certificate.

If speech is ever unavailable, `invisible-deck-helper.exe --speech-info` lists
every path it searched and marks which it found.

CI tests and builds on Ubuntu and Windows for every push
(`.github/workflows/ci.yml`), and tagging `v<version>` publishes both the
Flatpak and the Windows ZIP to the GitHub release
(`.github/workflows/release.yml`).

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
windows/build-portable-zip.sh       Windows portable ZIP, cross-built from Linux
windows/build-portable-zip.ps1      Windows portable ZIP, built natively on Windows
windows/README-portable.txt         the README shipped inside the ZIP
.github/workflows/ci.yml            test + build on Ubuntu and Windows
.github/workflows/release.yml       tag -> Flatpak + Windows ZIP on a release
```

## Licence

Invisible Deck Helper is free software: you can redistribute it and/or modify it
under the terms of the **GNU General Public License, version 3 or (at your
option) any later version**. See [LICENSE](LICENSE) for the full text.

Both packages bundle third-party software with its own licences — Piper TTS
(GPL-3.0-or-later), the espeak-ng data (GPL-3.0-or-later), CPython (PSF-2.0),
onnxruntime (MIT), numpy (BSD-3-Clause) and the `en_GB-alan-medium` voice
(CC BY 4.0). [NOTICE](NOTICE) lists each component and its licence; the
installer shows LICENSE during setup and both files ship inside the ZIP and the
installed program folder.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU General Public License for more details.
