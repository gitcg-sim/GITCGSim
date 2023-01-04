use enum_map::Enum;

use super::*;

use std::fs::File;
use std::io::{self, BufRead};

pub fn read_decklist_from_file(file: File) -> Result<Decklist, io::Error> {
    let lines = io::BufReader::new(file).lines();
    let mut lines_vec = vec![];
    for line in lines {
        lines_vec.push(line?);
    }
    Ok(Decklist::from_lines(lines_vec))
}

impl CharId {
    pub fn from_name(name: &str) -> Option<Self> {
        (0..Self::LENGTH).into_iter().find_map(|i| {
            let char_id = Self::from_usize(i);
            if char_id.get_char_card().name == name {
                Some(char_id)
            } else {
                None
            }
        })
    }
}

impl CardId {
    pub fn from_name(name: &str) -> Option<Self> {
        (0..Self::LENGTH).into_iter().find_map(|i| {
            let card_id = Self::from_usize(i);
            if card_id.get_card().name == name {
                Some(card_id)
            } else {
                None
            }
        })
    }
}

impl Decklist {
    pub fn from_lines(lines: Vec<String>) -> Self {
        let mut characters = smallvec![];
        let mut cards = smallvec![];
        let mut blank = false;
        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                blank = true;
                continue;
            }

            if blank {
                let Some(card_id) = CardId::from_name(line) else { continue };
                cards.push(card_id);
            } else {
                let Some(char_id) = CharId::from_name(line) else { continue };
                characters.push(char_id);
            }
        }

        Decklist { characters, cards }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_lines() {
        let lines = vec![
            " Yoimiya".to_string(),
            "Kamisato Ayaka  ".to_string(),
            "Diona".to_string(),
            " ".to_string(),
            "The Bestest Travel Companion!".to_string(),
            " Starsigns ".to_string(),
            "The Bestest Travel Companion!".to_string(),
            "Starsigns".to_string(),
            "Mushroom Pizza".to_string(),
            "Elemental Resonance: Woven Ice".to_string(),
            "Mushroom Pizza".to_string(),
            "Broken Rime's Echo".to_string(),
        ];
        assert_eq!(
            Decklist::new(
                smallvec![CharId::Yoimiya, CharId::KamisatoAyaka, CharId::Diona],
                smallvec![
                    CardId::TheBestestTravelCompanion,
                    CardId::Starsigns,
                    CardId::TheBestestTravelCompanion,
                    CardId::Starsigns,
                    CardId::MushroomPizza,
                    CardId::ElementalResonanceWovenIce,
                    CardId::MushroomPizza,
                    CardId::BrokenRimesEcho,
                ]
            ),
            Decklist::from_lines(lines)
        )
    }
}
