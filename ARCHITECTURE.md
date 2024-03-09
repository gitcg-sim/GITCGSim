# Architecture

# Overview game state evolution

```text
Dispatcher -> Commands -> Mutation
```

There are 3 layers of game state evolution: Dispatcher, command execution
and game state mutation.

## 1. The Dispatcher

The dispatcher is responsible for converting game actions (playing cards, casting
skills, etc) into invocations of card/effect implementations.

The card/effect implementations generate actions performed on the game state
through the command pattern.


## 2. Commands

Commands represent units of card effects such as:

- Deal DMG
- Heal active character
- Create a summon
- Apply a status
- Switch character

In addition, commands internal to the operation of TCG include:

- Trigger an event
- Hand over turn to another player

The command pattern allows card implementations to make changes to the game
state without a mutable borrow of the game state.

### Suspending execution

The execution of commands can be suspended to ask for an external input.
When that happens, the list of commands  must be remembered the game state.

Some causes of suspension are:

- When a character dies and the player must switch to a character of their choice
- When non-determinism is required (draw cards, create random summons, etc.)

## 3. Hashing and game state mutation

[Zobrist hashing](https://en.wikipedia.org/wiki/Zobrist_hashing) is
stored and kept track of in the game state as a way to quickly determine
if two game states are equal. Zobrist hashes are updated incrementally
when any hashable aspect of the game state is updated.

Do not update game state-related fields manually or else the hash consistency
will be violated.

## Card and effect implementations

Cards and effects are identified by enum types in the module `ids::enums`.

The programmatic aspects of cards are implemented in traits `CardImpl`, and `StatusImpl`s.

`StatusImpl`s are for:

- Character status
- Team status
- Artifact
- Weapon
- Summon
- Support

## Trigger Events

Event handlers are part of `StatusImpl`.

There are 2 kinds of events, `TriggerEvent` and `XEvent`.

`TriggerEvent` are identified by an enum `EventId`, and they
are mostly used for timings such as `EndPhase` and `EndOfTurn`.

An `XEvent` is used for events with additional information such as:

- DMG dealt/received
- The `SkillId` of the skill being cast
- Own skill vs. opponent skill being cast

Event handlers can affect the game state by adding commands to the
`TriggerEventContext::out_cmds`.

## Non-determinism

Non-determinism is not handled within the game state, instead a `NondetRequest`
is returned by the dispatcher and a `NondetResult` must be provided to continue
game state evolution.

The `GameStateWrapper` type allows `NondetRequest`s to be handled automatically
through an implementation of `NondetHandler`.

# Testing

## Unit tests

Unit tests test individual unit of the TCG simulator, such as dice management.

## Integration tests

Integration tests test the correctness of the TCG implementation in terms
of TCG rules and card effects.

## Property tests

Property tests test the correctness of the TCG implementation through
randomly generated data such as random game states.

The consistency of incrementally updated Zobrist hashes is one of the
properties being tested this way.

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