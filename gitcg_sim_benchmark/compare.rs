use gitcg_sim::deck::{random_decklist, Decklist};
use gitcg_sim::prelude::*;
use gitcg_sim::rand::{rngs::SmallRng, RngCore, SeedableRng};
use gitcg_sim::thiserror;
use gitcg_sim_cli_utils::cli_args::{GenericSearch, SearchConfig};
use gitcg_sim_search::SearchLimits;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use crate::iterate_match;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum DeckSrc {
    #[serde(rename = "random")]
    Random(u64),
    #[serde(rename = "file")]
    FromFile(String),
    #[serde(rename = "decklist")]
    Decklist(Decklist),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EntryConfig {
    pub name: String,
    pub deck: DeckSrc,
    pub search_config: SearchConfig,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CompareOpts {
    #[serde(default)]
    pub parallel: bool,
    #[serde(default = "CompareOpts::default_random_seed")]
    pub random_seed: u64,
    #[serde(default)]
    pub search_limits: Option<SearchLimits>,
    #[serde(default)]
    pub random_decks: bool,
    #[serde(default = "CompareOpts::default_match_rounds")]
    pub match_rounds: u32,
    #[serde(default = "CompareOpts::default_max_steps_per_round")]
    pub max_steps_per_round: u32,
    #[serde(default)]
    pub configs: Vec<EntryConfig>,
}

impl CompareOpts {
    fn default_random_seed() -> u64 {
        100
    }
    fn default_match_rounds() -> u32 {
        100
    }
    fn default_max_steps_per_round() -> u32 {
        200
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseCompareOptsError {
    #[error("failed to load file")]
    FileLoadError(#[from] std::io::Error),
    #[error("failed to parse JSON")]
    DeserializeError(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ConstructEntryError {
    #[error("failed to load decklist")]
    LoadDecklistError(#[from] std::io::Error),
}

impl EntryConfig {
    pub fn construct<S: NondetState>(
        &self,
        parallel: bool,
        limits: Option<SearchLimits>,
    ) -> Result<(Decklist, impl '_ + Fn() -> GenericSearch<S>), ConstructEntryError> {
        let decklist = self.deck.decklist()?;
        let make_search = move || self.search_config.make_search(parallel, limits);
        Ok((decklist, make_search))
    }
}

impl DeckSrc {
    pub fn decklist(&self) -> Result<Decklist, std::io::Error> {
        match self {
            Self::Random(seed) => Ok(random_decklist(&mut SmallRng::seed_from_u64(*seed))),
            Self::FromFile(path) => {
                let file = File::open(path)?;
                let reader = BufReader::new(file);
                let lines_res: Result<Vec<_>, _> = reader.lines().collect();
                Ok(Decklist::from_lines(lines_res?))
            }
            Self::Decklist(deck) => Ok(deck.clone()),
        }
    }
}

pub fn parse_compare_opts(json_path: &PathBuf) -> Result<CompareOpts, ParseCompareOptsError> {
    Ok(serde_json::from_reader(BufReader::new(File::open(json_path)?))?)
}

fn standard_game(decks: ByPlayer<&Decklist>, mut rng: SmallRng, random_decks: bool) -> GameStateWrapper {
    // TODO use ByPlayer in the call tree including new_standard_game
    let (d1, d2) = if random_decks {
        (random_decklist(&mut rng), random_decklist(&mut rng))
    } else {
        (decks.0.clone(), decks.1.clone())
    };
    let rng = SmallRng::seed_from_u64(rng.next_u64());
    gitcg_sim::prelude::new_standard_game(&d1, &d2, rng)
}

pub fn main_compare(opts: CompareOpts) -> Result<(), String> {
    let parallel = opts.parallel;
    let limits = opts.search_limits;
    let results: Result<Vec<_>, _> = opts
        .configs
        .iter()
        .map(move |c| {
            c.construct::<StandardNondetHandlerState>(parallel, limits)
                .map(|x| (&c.name, x))
        })
        .collect();
    let entries = results.map_err(|e| e.to_string())?;
    let n = entries.len();
    let mut matchup: Vec<Vec<f32>> = (0..n).map(|_| (0..n).map(|_| 0f32).collect()).collect();
    let random_decks = opts.random_decks;
    let rounds = opts.match_rounds;
    let steps = opts.max_steps_per_round;
    let random_seed = opts.random_seed;
    for (i, row) in matchup.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            if j <= i {
                continue;
            }
            let cs = ByPlayer::new(&entries[i], &entries[j]);
            let decks = cs.map(|(_, (x, _))| x);
            println!("--- {} vs. {}", entries[i].0, entries[j].0);
            // dbg!(&decks);
            let (_, score, _) = iterate_match(
                &|| cs.map(|(_, x)| x.1()),
                &|rng| standard_game(decks, rng, random_decks),
                crate::IterateMatchOpts {
                    rounds,
                    steps,
                    random_seed,
                },
            );
            *cell = score;
        }
    }
    for row in matchup {
        for cell in row {
            print!("{cell:4} ");
        }
        println!();
    }
    Ok(())
}
