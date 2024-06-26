# Genius Invokation TCG Simulator

A library for the [Genius Invokation TCG](https://genshin-impact.fandom.com/wiki/Genius_Invokation_TCG).

## Summary of crates in this workspace

### `gitcg_sim`
- `no_std` support
- Game state representation and evolution
- Move generation and validation
- Game state determinization (handling unknown/random information)

### `gitcg_sim_code_gen`
- Used by `gitcg_sim` for code generation

### `gitcg_sim_search`
- Minimax and MCTS (Monte-Carlo Tree Search) algorithms

### `gitcg_sim_cli_utils`
- Command line argument parsing for `gitcg_sim_benchmark` and `gitcg_sim_tui_app`

### `gitcg_sim_benchmark`
- Performance test for move generation
- Random playout

### `gitcg_sim_tui_app`
- TUI-based GITCG simulator

## Unsupported TCG features

- Mulligan at game start
- Manually selecting Elemental Dice for cost payments (auto automatically)
- Manually selecting Elemental Dice for rerolling (done automatically)
- Validation rules for decklists (Elemental Resonance, talent cards)

## Adding new characters or cards

The cards and effects are **hard-coded** in the `gitcg_sim` crate.
To add new cards you must modify its source code.

## Running the simulator

```bash
# Run the terminal-based simulator
cargo run --release

# Run the benchmark
cd ./gitcg_sim_benchmark
cargo run --release
```

### WebAssembly-based simulator

Visit <https://gitcgsimwebdemo.netlify.app/> for a demo.

### Command line simulator

Controls:

 - `Up`/`Down` or `PageUp`/`PageDown`: Scroll through the log
 - `j`, `k`: Move up/down in the actions list
 - `Enter`: Perform the selected action
 - `1` - `9` and `0`: Perform the 1st - 10th action
 - `q`: Exit

![tui_app_screenshot](./tui_app_screenshot.png)

### Common command line flags

#### TCG
 - `--random-decks`: Randomly generate decks for both players
 - `--player1-deck, --player2-deck:` Relative paths to the player's decks

#### Search
 - `--algorithm [minimax|mcts]`: Game tree search algorithm
 - `-P/--parallel`: Enable parallelism
 - `--tt-size-mb`: Transposition table size in megabytes
 - `-T/--time-limit-ms`: Time limit per move in milliseconds

### Executables

#### `gitcg_sim_benchmark`
Runs a computer vs. computer simulation of the TCG.

Subcommands:

 - `speedup`: Compare the performances of sequential and parallel searches
 - `benchmark`: Run a single simulation - `match`: Measure the win rate between a configured search algorithm vs.
    a standarized search algorithm (configured with `--standard-algorithm` and `--standard-time-limit-ms`)

#### `gitcg_sim_tui_app`
Runs the command line simulator vs. a computer opponent.

#### `gitcg_sim_self_play`
Runs the trainer for the policy or evaluation networks.

## Feature flags

By default, all features are disabled.

### `serde`
Enables `serde` support for the relevant datatypes.

### `wasm`
Required for WebAssembly builds. Also enables `serde` and `no_parallel`.

### `no_parallel`
Disables parallelization thorugh the `rayon` dependency.

### `no_static_status_impl`
Use dynamic dispatch (`dyn StatusImpl`) instead of heavily
inlined trait implementations for dispatching `StatusImpl`s.

### `training`
Enable machine learning dependencies (`dfdx` and `ndarray`) and allow
loading policy networks from .npz files.
Enabled by the policy network training executable.

## The `GameState` type

### Creation
To construct a [`GameState`](crate::prelude::GameState), use the [`GameStateInitializer`](crate::prelude::GameStateInitializer) type.

The [`new_standard_game`](crate::prelude::new_standard_game) function bypass most intermediate steps for constructing a `GameStateWrapper`.

```rust
use gitcg_sim::prelude::*;
use gitcg_sim::{vector, list8};
use gitcg_sim::rand::{rngs::SmallRng, SeedableRng}; // Re-exports of rand crate

let deck1 = Decklist::new(vector![CharId::Yoimiya, CharId::KamisatoAyaka, CharId::Bennett], vec![/* CardId::... */].into());
let deck2 = Decklist::new(vector![CharId::Fischl, CharId::RhodeiaOfLoch, CharId::FatuiPyroAgent], vec![].into());
let rng = SmallRng::seed_from_u64(100).into();
let game_state_wrapper = new_standard_game((&deck1, &deck2).into(), rng);
```

### State evolution
To advance a `GameState`, call [`GameState::advance`](crate::prelude::GameState::advance).
To get a list of actions, [`GameState::available_actions`](crate::prelude::GameState::available_actions).

If `GameState::advance` returns an `Err(..)`, then the game state is invalidated (cost payments are not reversed, for example.).

```rust
use gitcg_sim::prelude::*;

// Create a new GameState
let mut game_state: GameState = GameStateInitializer::default()
    .characters(
       vec![CharId::Yoimiya, CharId::KamisatoAyaka, CharId::Bennett],
       vec![CharId::Fischl, CharId::RhodeiaOfLoch, CharId::FatuiPyroAgent],
    )
    .start_at_beginning()
    .build();

// Waiting for nondeterministic input, so no player is to move
assert_eq!(None, game_state.to_move_player());

// Add cards to both players hands
game_state
    .advance(Input::NondetResult(NondetResult::ProvideCards(
        // .into() constructs an instance of ByPlayer from a tuple
        (
            [CardId::LeaveItToMe, CardId::Starsigns].into(),
            [CardId::Strategize, CardId::Paimon].into(),
        ).into()
    )))
    .unwrap();

// Select starting characters
game_state
    .advance(Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)))
    .unwrap();

game_state
    .advance(Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(0)))
    .unwrap();

// Advance Roll Phase
game_state.advance(Input::NoAction).unwrap();

// Add 8 Omni dice to both players
game_state
    .advance(Input::NondetResult(NondetResult::ProvideDice((
        DiceCounter::omni(8),
        DiceCounter::omni(8),
    ).into())))
    .unwrap();

println!("{:?}", game_state.available_actions());
```

### The `Input` type

Input provided to advancing the `GameState`, both [deterministic](crate::prelude::Input::FromPlayer) and
[non-deterministic](crate::prelude::Input::NondetResult).

See [`Input`](crate::prelude::Input).

### Hashing and mutation

The game state is hashed incrementally through [Zobrist hashing](https://www.chessprogramming.org/Zobrist_Hashing).
If the game state is updated manually outside of `advance`,
[`game_state.rehash()`](crate::prelude::GameState::rehash) must be called to recopmute the hash.

```rust
use gitcg_sim::prelude::*;

// Create a new GameState
let mut game_state: GameState = GameStateInitializer::default()
    .characters(
       vec![CharId::Yoimiya, CharId::KamisatoAyaka, CharId::Bennett],
       vec![CharId::Fischl, CharId::RhodeiaOfLoch, CharId::FatuiPyroAgent],
    )
    // Bypass select starting character and mulligan
    .skip_to_roll_phase()
    .build();

// Get the Zobrist hash
game_state.zobrist_hash();
// Perform an external update
// game_state.advance(...);
// Recalculate the hash
game_state.rehash();
```

### Handling non-determinism

The [`GameStateWrapper`](crate::prelude::GameStateWrapper) type handles non-determinism automatically using a player decks and an existing RNG.

```rust
use gitcg_sim::prelude::*;
use gitcg_sim::rand::{rngs::SmallRng, SeedableRng}; // Re-exports of rand crate

let deck1 = Decklist::new(
    vec![CharId::Yoimiya, CharId::KamisatoAyaka, CharId::Bennett].into(),
    vec![/* CardId::... */].into()
);
let deck2 = Decklist::new(
    vec![CharId::Fischl, CharId::RhodeiaOfLoch, CharId::FatuiPyroAgent].into(),
    vec![].into()
);
let rng = SmallRng::seed_from_u64(100).into();

// Nondet provider based on deck and RNG
let nd = NondetProvider::new(StandardNondetHandlerState::new(&deck1, &deck2, rng));
// This nondet provider that does nothing
// let nd_state = NondetProvider::new(EmptyNondetState());

let game_state: GameState = GameStateInitializer::default()
    .characters(deck1.characters, deck2.characters)
    .skip_to_roll_phase()
    .build();
let game_state_wrapper = GameStateWrapper::new(game_state, nd);
```

### Serialization and deserialization

Enable the `serde` feature to serialize and deserialize the relevant types.