use super::*;

use enum_map::Enum;

impl CharId {
    pub fn from_name(name: &str) -> Option<Self> {
        (0..Self::LENGTH).find_map(|i| {
            let char_id = Self::from_usize(i);
            (char_id.char_card().name == name).then_some(char_id)
        })
    }
}

impl CardId {
    pub fn from_name(name: &str) -> Option<Self> {
        (0..Self::LENGTH).find_map(|i| {
            let card_id = Self::from_usize(i);
            (card_id.card().name == name).then_some(card_id)
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
                let Some(card_id) = CardId::from_name(line) else {
                    continue;
                };
                cards.push(card_id);
            } else {
                let Some(char_id) = CharId::from_name(line) else {
                    continue;
                };
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
