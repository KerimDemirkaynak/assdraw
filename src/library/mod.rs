//! Shape library — port of library.hpp/cpp (preview list, default shapes, save/load to canvas).

use crate::engine::{parse_ass_into, Drawing};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub name: String,
    pub drawing: Drawing,
    pub source_ass: String,
    #[serde(default)]
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeLibrary {
    pub entries: Vec<LibraryEntry>,
}

impl Default for ShapeLibrary {
    fn default() -> Self {
        let mut lib = Self {
            entries: Vec::new(),
        };
        lib.add_defaults();
        lib
    }
}

impl ShapeLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: impl Into<String>, drawing: Drawing) {
        let source_ass = drawing.to_ass_string();
        self.entries.push(LibraryEntry {
            name: name.into(),
            drawing,
            source_ass,
            selected: false,
        });
    }

    pub fn get(&self, idx: usize) -> Option<&LibraryEntry> {
        self.entries.get(idx)
    }

    pub fn remove(&mut self, idx: usize) {
        if idx < self.entries.len() {
            self.entries.remove(idx);
        }
    }

    pub fn select_all(&mut self) {
        for e in &mut self.entries {
            e.selected = true;
        }
    }

    pub fn select_none(&mut self) {
        for e in &mut self.entries {
            e.selected = false;
        }
    }

    pub fn delete_selected(&mut self) {
        self.entries.retain(|e| !e.selected);
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    fn add_defaults(&mut self) {
        // Star (centered around 0,0)
        let mut star = Drawing::new();
        let _ = parse_ass_into(
            &mut star,
            "m 0 -30 l 9 -9 l 30 -9 l 13 4 l 19 25 l 0 12 l -19 25 l -13 4 l -30 -9 l -9 -9",
        );
        self.add("Yıldız (Star)", star);

        // Triangle / Arrow head (from ASSDraw3)
        let mut tri = Drawing::new();
        let _ = parse_ass_into(&mut tri, "m 8 -14 l 36 -25 l -39 -25");
        self.add("Üçgen (Triangle)", tri);

        // Heart (centered around 0,0)
        let mut heart = Drawing::new();
        let _ = parse_ass_into(
            &mut heart,
            "m 0 25 b -28 5 -36 -18 -18 -28 b 0 -28 0 -8 0 -5 b 0 -8 0 -28 18 -28 b 36 -18 28 5 0 25",
        );
        self.add("Kalp (Heart)", heart);

        // Arrow Right (centered around 0,0)
        let mut arrow = Drawing::new();
        let _ = parse_ass_into(
            &mut arrow,
            "m -30 -10 l 5 -10 l 5 -20 l 30 0 l 5 20 l 5 10 l -30 10",
        );
        self.add("Ok (Arrow)", arrow);

        // Speech Bubble (centered around 0,0)
        let mut bubble = Drawing::new();
        let _ = parse_ass_into(
            &mut bubble,
            "m -25 -20 l 25 -20 b 32 -20 35 -16 35 -10 l 35 10 b 35 16 32 20 25 20 l 5 20 l -15 32 l -5 20 l -25 20 b -32 20 -35 16 -35 10 l -35 -10 b -35 -16 -32 -20 -25 -20",
        );
        self.add("Konuşma Balonu (Bubble)", bubble);

        // Diamond (centered around 0,0)
        let mut diamond = Drawing::new();
        let _ = parse_ass_into(&mut diamond, "m 0 -25 l 25 0 0 25 -25 0");
        self.add("Elmas (Diamond)", diamond);
    }
}
