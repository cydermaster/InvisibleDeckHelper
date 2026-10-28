Invisible Deck Helper __VERSION__ (Windows 11, 64-bit, portable)
==========================================================

A practice aid for the Invisible Deck card trick. It deals one card at a
time and speaks it aloud with Piper, using the English (Great Britain)
Alan voice in medium quality. Everything runs on your machine; nothing is
sent anywhere.

This folder is self-contained. There is nothing to install.


RUNNING IT
----------
Double-click  invisible-deck-helper.exe

The first launch may show a blue "Windows protected your PC" box, because
this build is not code-signed. Click "More info", then "Run anyway". That
warning is about the missing signature, not about the program's behaviour.

Keep the whole folder together: the exe needs the "python" and "voices"
folders beside it. Copy the folder anywhere (Desktop, a USB stick) as a unit.


USING IT
--------
Deal new card ....... click the button, or press Space / Enter
Show pair ........... tick it to reveal the invisible partner card
Say again ........... repeat the announcement
Mute speech ......... tick for silence; the window still shows
                      "Muted - would say: ..." so you can read it

With the pair toggle OFF it says just the card, e.g. "Queen of Hearts".
With it ON it says "Queen of Hearts pairs with Ace of Spades".

The pairings are the standard Invisible Deck scheme: Hearts pair with
Spades, Diamonds pair with Clubs, each pair totals 13, and Kings pair
with Kings. Run  invisible-deck-helper.exe --pairs  to see all 26.


COMMAND LINE EXTRAS
-------------------
invisible-deck-helper.exe --print [--pair] [--seed N]
invisible-deck-helper.exe --pairs
invisible-deck-helper.exe --speech-info
invisible-deck-helper.exe --say "Ten of Spades pairs with Three of Hearts"
invisible-deck-helper.exe --version
invisible-deck-helper.exe --help

Set IDH_MUTE=1 to start with speech muted.
Set IDH_PIPER=<path> to use a different piper executable.


IF IT DOES NOT SPEAK
--------------------
--speech-info reports which engine and voice it found. The app searches
for python\python.exe next to the exe, then a piper.exe beside it, then
piper on PATH. Rebuilding the "python" folder from the source tree fixes a
damaged copy:  windows/build-portable-zip.sh

Speech synthesis takes a second or two per phrase, because the voice model
is loaded fresh each time.

Version __VERSION__ - https://github.com/cydermaster/InvisibleDeckHelper
See LICENSE for the project licence and NOTICE for the bundled components.
