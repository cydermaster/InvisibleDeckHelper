//! Invisible Deck Helper — a practice aid for the Invisible Deck card trick.
//!
//! Deals one card at a time and speaks it aloud with Piper (en_GB-alan-medium).
//! By default it says just the card ("Queen of Hearts"); with the pair toggle on
//! it says "Queen of Hearts pairs with Ace of Spades".

mod cards;
mod rng;
mod speech;

use cards::{all_pairs, full_deck, Card, RANK_SHORT};
use eframe::egui;
use rng::Random;

const APP_NAME: &str = "Invisible Deck Helper";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn card_ink(card: Card) -> egui::Color32 {
    if card.suit.is_red() {
        egui::Color32::from_rgb(192, 57, 43)
    } else {
        egui::Color32::from_rgb(27, 27, 27)
    }
}

/// Paint a single face-up card.
fn draw_card(ui: &mut egui::Ui, card: Card, size: egui::Vec2) {
    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();

    let rounding = egui::Rounding::same(12.0);
    painter.rect_filled(rect, rounding, egui::Color32::from_rgb(251, 251, 244));
    painter.rect_stroke(
        rect,
        rounding,
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(195, 195, 182)),
    );
    painter.rect_stroke(
        rect.shrink(6.0),
        egui::Rounding::same(8.0),
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(222, 222, 212)),
    );

    let rank = RANK_SHORT[(card.rank - 1) as usize];
    painter.text(
        rect.left_top() + egui::vec2(14.0, 10.0),
        egui::Align2::LEFT_TOP,
        format!("{rank}{}", card.suit.symbol()),
        egui::FontId::proportional(30.0),
        card_ink(card),
    );
    painter.text(
        rect.center() + egui::vec2(0.0, 14.0),
        egui::Align2::CENTER_CENTER,
        card.suit.symbol(),
        egui::FontId::proportional(size.y * 0.42),
        card_ink(card),
    );
}

/// The sentence spoken for a given card and toggle state.
fn spoken_phrase(card: Card, show_pair: bool) -> String {
    if show_pair {
        format!("{} pairs with {}", card.long(), card.partner().long())
    } else {
        card.long()
    }
}

struct HelperApp {
    current: Card,
    show_pair: bool,
    random: Random,
    deals: u32,
    speaker: Option<speech::Speaker>,
    speech_status: String,
    last_phrase: String,
}

impl HelperApp {
    fn new() -> Self {
        let mut random = Random::live();
        let deck = full_deck();
        let current = deck[random.below(deck.len())];
        let (speaker, speech_status) = match speech::Speaker::detect() {
            Some(s) => {
                let desc = s.describe();
                (Some(s), format!("speech: {desc}"))
            }
            None => (
                None,
                "speech unavailable (piper or voice model not found) — dealing silently".to_string(),
            ),
        };
        let mut app = Self {
            current,
            show_pair: false,
            random,
            deals: 1,
            speaker,
            speech_status,
            last_phrase: String::new(),
        };
        app.speak_current();
        app
    }

    /// Speak the current card, honouring the pair toggle. Speech runs on its own
    /// thread so synthesis never blocks the window.
    fn speak_current(&mut self) {
        let phrase = spoken_phrase(self.current, self.show_pair);
        self.last_phrase = phrase.clone();
        if let Some(speaker) = &self.speaker {
            let speaker = speaker.clone();
            std::thread::spawn(move || {
                if let Err(e) = speaker.say(&phrase) {
                    eprintln!("speech failed: {e}");
                }
            });
        }
    }

    fn deal(&mut self) {
        let deck = full_deck();
        self.current = deck[self.random.below(deck.len())];
        self.deals = self.deals.saturating_add(1);
        self.speak_current();
    }
}

impl eframe::App for HelperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Space deals a new card, which is the one thing you want to do repeatedly.
        if ctx.input(|i| i.key_pressed(egui::Key::Space) || i.key_pressed(egui::Key::Enter)) {
            self.deal();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading(APP_NAME);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("v{VERSION}"))
                            .small()
                            .color(egui::Color32::from_gray(130)),
                    );
                });
            });
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("Think of a card, then check yourself against the deck.")
                    .color(egui::Color32::from_gray(140)),
            );

            ui.add_space(14.0);

            // ---- the card(s) ------------------------------------------------
            let card_size = egui::vec2(210.0, 294.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Card").strong());
                    draw_card(ui, self.current, card_size);
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(self.current.long()).strong());
                });

                ui.add_space(20.0);

                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Invisible pair").strong());
                    if self.show_pair {
                        draw_card(ui, self.current.partner(), card_size);
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new(self.current.partner().long()).strong());
                    } else {
                        // A face-down card back, so the layout does not jump.
                        let (rect, _r) = ui.allocate_exact_size(card_size, egui::Sense::hover());
                        let painter = ui.painter();
                        painter.rect_filled(
                            rect,
                            egui::Rounding::same(12.0),
                            egui::Color32::from_rgb(23, 81, 63),
                        );
                        painter.rect_stroke(
                            rect.shrink(6.0),
                            egui::Rounding::same(8.0),
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(40, 120, 95)),
                        );
                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "?",
                            egui::FontId::proportional(64.0),
                            egui::Color32::from_rgb(120, 180, 158),
                        );
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new("hidden").color(egui::Color32::from_gray(120)));
                    }
                });
            });

            ui.add_space(16.0);

            // ---- controls ---------------------------------------------------
            ui.horizontal(|ui| {
                if ui.button(egui::RichText::new("Deal new card").size(16.0)).clicked() {
                    self.deal();
                }
                ui.add_space(12.0);
                let toggled = ui
                    .checkbox(&mut self.show_pair, "Show Invisible Deck pair")
                    .changed();
                if toggled {
                    // Speak the phrase that now applies, so the toggle is audible.
                    self.speak_current();
                }
                ui.add_space(12.0);
                if ui.button("Say again").clicked() {
                    self.speak_current();
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            ui.label(
                egui::RichText::new(format!(
                    "Cards dealt: {}.  Space or Enter deals again.",
                    self.deals
                ))
                .small()
                .color(egui::Color32::from_gray(140)),
            );
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(format!("Saying: \"{}\"", self.last_phrase))
                    .small()
                    .color(egui::Color32::from_gray(160)),
            );
            ui.label(
                egui::RichText::new(&self.speech_status)
                    .small()
                    .color(egui::Color32::from_gray(120)),
            );
        });
    }
}

// ---------------------------------------------------------------------------
// Headless modes (used by tests, packaging smoke checks, and terminals)
// ---------------------------------------------------------------------------

fn print_pairs() {
    println!("Invisible Deck pairings ({} pairs)", all_pairs().len());
    println!("Hearts pair with Spades; Diamonds pair with Clubs; pairs total 13; Kings pair with Kings.");
    println!();
    for (a, b) in all_pairs() {
        println!("  {:>3}  {:>3}   {}  /  {}", a.short(), b.short(), a.long(), b.long());
    }
}

fn print_one(seed: Option<u64>, show_pair: bool) {
    let deck = full_deck();
    let mut random = match seed {
        Some(s) => Random::seeded(s),
        None => Random::live(),
    };
    let card = deck[random.below(deck.len())];
    println!("card   : {} ({})", card.short(), card.long());
    println!("pair   : {} ({})", card.partner().short(), card.partner().long());
    println!("phrase : \"{}\"", spoken_phrase(card, show_pair));
}

fn help() {
    println!("{APP_NAME} {VERSION}");
    println!();
    println!("USAGE:");
    println!("    invisible-deck-helper [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --print [--seed N] [--pair]   Print one card, its partner and the spoken phrase");
    println!("    --pairs                       Print all 26 pairings");
    println!("    --say \"<text>\"                Speak the text with Piper and exit");
    println!("    --speech-info                 Report the TTS engine and voice in use");
    println!("    --version                     Print version");
    println!("    -h, --help                    Print this help");
    println!();
    println!("With no options the window opens and speaks one card at a time.");
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |f: &str| args.iter().any(|a| a == f);

    if has("--version") {
        println!("{APP_NAME} {VERSION}");
        return Ok(());
    }
    if has("--help") || has("-h") {
        help();
        return Ok(());
    }
    if has("--pairs") {
        print_pairs();
        return Ok(());
    }
    if has("--speech-info") {
        match speech::Speaker::detect() {
            Some(s) => println!("speech: available — {}", s.describe()),
            None => println!("speech: unavailable (piper or voice model not found)"),
        }
        return Ok(());
    }
    if let Some(i) = args.iter().position(|a| a == "--say") {
        let text = args.get(i + 1).cloned().unwrap_or_default();
        if text.is_empty() {
            eprintln!("error: --say needs some text");
            std::process::exit(2);
        }
        let Some(speaker) = speech::Speaker::detect() else {
            eprintln!("speech unavailable: piper or voice model not found");
            std::process::exit(1);
        };
        // Synthesise and report, then try to play. Reported separately so a
        // machine without an audio device can still prove the TTS path works.
        let out = std::env::temp_dir().join(format!("idh-cli-{}.wav", std::process::id()));
        if let Err(e) = speaker.synthesise(&text, &out) {
            eprintln!("synthesis failed: {e}");
            std::process::exit(1);
        }
        let size = std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0);
        println!("synthesised \"{text}\" -> {size} bytes via {}", speaker.describe());
        match speech::play_wav(&out) {
            Ok(()) => println!("played"),
            Err(e) => println!("playback unavailable: {e}"),
        }
        let _ = std::fs::remove_file(&out);
        return Ok(());
    }
    if has("--print") {
        let seed = args
            .iter()
            .position(|a| a == "--seed")
            .and_then(|i| args.get(i + 1))
            .map(|s| s.parse::<u64>().expect("--seed must be a non-negative integer"));
        print_one(seed, has("--pair"));
        return Ok(());
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([540.0, 740.0])
            .with_min_inner_size([460.0, 660.0])
            .with_title(APP_NAME),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|_cc| Ok(Box::new(HelperApp::new()) as Box<dyn eframe::App>)),
    )
}
