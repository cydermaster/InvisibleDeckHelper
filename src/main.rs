//! Invisible Deck Helper — a practice aid for the Invisible Deck card trick.
//!
//! Shows one card at a time. The spectator "names a card"; you turn over the
//! deck, spread it, and the named card is the only face-up one. This helper
//! deals you a random card to practise against, and an optional toggle reveals
//! the invisible partner it is glued to — off by default, so you can self-test.

mod cards;
mod rng;

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

struct HelperApp {
    current: Card,
    show_pair: bool,
    random: Random,
    deals: u32,
}

impl HelperApp {
    fn new() -> Self {
        let mut random = Random::live();
        let deck = full_deck();
        let current = deck[random.below(deck.len())];
        Self { current, show_pair: false, random, deals: 1 }
    }

    fn deal(&mut self) {
        let deck = full_deck();
        self.current = deck[self.random.below(deck.len())];
        self.deals = self.deals.saturating_add(1);
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
                ui.checkbox(&mut self.show_pair, "Show Invisible Deck pair");
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

fn print_one(seed: Option<u64>) {
    let deck = full_deck();
    let mut random = match seed {
        Some(s) => Random::seeded(s),
        None => Random::live(),
    };
    let card = deck[random.below(deck.len())];
    println!("card  : {} ({})", card.short(), card.long());
    println!("pair  : {} ({})", card.partner().short(), card.partner().long());
}

fn help() {
    println!("{APP_NAME} {VERSION}");
    println!();
    println!("USAGE:");
    println!("    invisible-deck-helper [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --print [--seed N]   Print one card and its invisible partner, then exit");
    println!("    --pairs              Print all 26 pairings, then exit");
    println!("    --version            Print version, then exit");
    println!("    -h, --help           Print this help, then exit");
    println!();
    println!("With no options the window opens showing one card at a time.");
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--version") {
        println!("{APP_NAME} {VERSION}");
        return Ok(());
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        help();
        return Ok(());
    }
    if args.iter().any(|a| a == "--pairs") {
        print_pairs();
        return Ok(());
    }
    if args.iter().any(|a| a == "--print") {
        let seed = args
            .iter()
            .position(|a| a == "--seed")
            .and_then(|i| args.get(i + 1))
            .map(|s| s.parse::<u64>().expect("--seed must be a non-negative integer"));
        print_one(seed);
        return Ok(());
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 700.0])
            .with_min_inner_size([460.0, 640.0])
            .with_title(APP_NAME),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|_cc| Ok(Box::new(HelperApp::new()) as Box<dyn eframe::App>)),
    )
}
