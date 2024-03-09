# Architecture

# Code map of the `gitcg_sim` crate

## Helpers for the Genius Invokation TCG domain

### `src/data_structures`

Collection datatypes for the Genius Invokation TCG simulator.

### Re-exports for collection types
Third party collection types used:
 - `heapless`
 - `smallvec`
 - `enum_map`
 - `enumset`

#### `src/data_structures/capped_list`

This module contains the `CappedLengthList8<T, N>` type, which is similar to
the `heapless::Vec` except it supports `Copy` and contains no unsafe code.

### `src/dice_counter`

Module containing the implementation for the Elemental Dice portion of the game state.

Auto selection for paying and rerolling dice are also implemented there.

The `DiceCounter` represents Elemental Dice and the `ElementPriority` represents
elements prioritized for paying and rerolling.

### `src/reaction`

Logic for Genius Invokation TCG Elemental Reactions.

### `src/tcg_model`

Module containing domain-specific value types of the Genius Invokation TCG.

- Dealing DMG:
  - `DealDMG`
  - `DealDMGType`: `Piercing`, `Physical`, `Elemental(..)`
- Elements:
  - `Element`
  - `Reaction`
- Elemental Dice
  - `Dice`: `Omni`, `Element(..)`
- Card information:
  - `WeaponType`
  - `SkillType`
  - etc.

## Required for adding a new card (character/event/etc.)
### `src/ids`

The ID types and card lookup functionality.

#### `src/ids/enums`

Contains the definitions of ID enums. To define a new entity (character/status/etc.)
new entries must be added to the correspnding enums and then the corresponding module
must be added to the `cards::*::*` modules to be detected by the generated code.

ID enums:

- `CharId`
- `SkillId`
- `StatusId`
- `SummonId`
- `SupportId`
- `CardId`

##### Examples of mapping
- `CharId::Yoimiya` maps to `cards::characters::yoimiya`
- `StatusId::NiwabiEnshou` maps to `cards::characters::yoimiya::niwabi_enshou`
- `CardId::CalxsArts` maps to `cards::event::calxs_arts`
- `SummonId::BurningFlame` maps to `cards::summons::burning_flame`

### `src/cards`

This module the definitions of the Genius Invokation TCG cards.

 - `src/cards/characters`: Characters
 - `src/cards/equipment`: Equipment cards
 - `src/cards/event`: Event cards
 - `src/cards/support`: Support cards
 - `src/cards/statuses`: Applied effect statuses

## Game state representation and evolution

The game state is represented in a perfect-information and deterministic fashion.

### `src/types`

Types for representing the game state:

 - `GameState`
 - `PlayerState`
 - `CharState`
 - `StatusCollection`

Types for commands:

 - `Command`
 - `CommandContext`

Types for applied effect status trait and implementations:

 - `StatusImpl`
 - `RespondsTo`
 - `EventType`
 - `XEventType`

### `src/card_impls`

Event handlers

### `src/zobrist_hash`

### `src/dispatcher`

Contains the dispatcher, which dispatches player or non-deterministic inputs and evolves the game state.

### `src/dispatcher_ops`

Helpers for the dispatcher.

## Zobrist hashing

### `src/zobrist_hash`