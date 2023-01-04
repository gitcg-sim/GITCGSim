use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::vector;

use crate::data_structures::Vector;

use crate::cards::ids::*;

use super::command::*;
use super::deal_dmg::DealDMGType;
use super::enums::EquipSlot;
use super::input::{Input, PlayerAction};
use super::{
    card_defs::Cost,
    deal_dmg::DealDMG,
    enums::{Element, Reaction},
    game_state::*,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DMGSource {
    Summon(PlayerId, SummonId),
    Character(PlayerId, (u8, CharId)),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Event {
    Round(u8, PlayerId),
    Phase(Phase),
    Action(Input),
    PayCost(PlayerId, Cost, CostType),
    TriggerEvent(CommandSource, EventId),
    Summon(PlayerId, SummonId),
    ApplyTeamStatus(PlayerId, StatusId),
    ApplyCharStatus(PlayerId, (u8, CharId), StatusId),
    Equip(PlayerId, (u8, CharId), EquipSlot, Option<StatusId>),
    DealDMG(Option<DMGSource>, (PlayerId, (u8, CharId)), DealDMG),
    Heal(PlayerId, (u8, CharId), u8),
    ElemApplied(PlayerId, (u8, CharId), Element),
    Reaction(PlayerId, (u8, CharId), Reaction),
    CharacterDied(PlayerId, (u8, CharId)),
}

impl Event {
    pub fn indent_level(&self) -> u8 {
        match self {
            Event::Round(..) => 0,
            Event::Phase(..) => 0,
            Event::Action(..) => 1,
            Event::PayCost(..) => 2,
            Event::TriggerEvent(..) => 2,
            Event::Summon(..) => 3,
            Event::ApplyTeamStatus(..) => 3,
            Event::ApplyCharStatus(..) => 3,
            Event::Equip(..) => 3,
            Event::DealDMG(..) => 2,
            Event::Heal(..) => 2,
            Event::ElemApplied(..) => 3,
            Event::Reaction(..) => 3,
            Event::CharacterDied(..) => 2,
        }
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Round(r, p) => f.write_fmt(format_args!("Round {r}: {p} to move first")),
            Event::Phase(p) => {
                let s = match p {
                    Phase::RollPhase { .. } => "Roll Phase",
                    Phase::ActionPhase { .. } => "Action Phase",
                    Phase::EndPhase { .. } => "End Phase",
                    Phase::WinnerDecided { .. } => "Winner Decided",
                };
                f.write_str(s)
            }
            Event::Action(act) => match act {
                Input::FromPlayer(p, a) => match a {
                    PlayerAction::EndRound => f.write_fmt(format_args!("{p} ends their Round")),
                    PlayerAction::PlayCard(c, _) => {
                        f.write_fmt(format_args!("{p} played a Card: {}", c.get_card().name))
                    }
                    PlayerAction::ElementalTuning(_) => f.write_fmt(format_args!("{p} performed Elemental Tuning")),
                    PlayerAction::CastSkill(s) => f.write_fmt(format_args!("{p} used Skill: {}", s.get_skill().name)),
                    PlayerAction::PostDeathSwitch(c) | PlayerAction::SwitchCharacter(c) => {
                        f.write_fmt(format_args!("{p} switched character to: {c}"))
                    }
                },
                Input::NoAction | Input::NondetResult(_) => Ok(()),
            },
            Event::DealDMG(_, (tgt_player, (_, char_id)), dmg) => {
                let dmg_val = dmg.dmg;
                let char_name = char_id.get_char_card().name;
                f.write_fmt(format_args!("{tgt_player} {char_name} received {dmg_val}"))?;
                match dmg.dmg_type {
                    DealDMGType::Piercing => f.write_str(" Piercing DMG")?,
                    DealDMGType::Physical => f.write_str(" Physical DMG")?,
                    DealDMGType::Elemental(e) => f.write_fmt(format_args!(" {e:?} DMG"))?,
                };
                if dmg.piercing_dmg_to_standby > 0 {
                    let pd = dmg.piercing_dmg_to_standby;
                    f.write_fmt(format_args!(" + {pd} Piercing DMG on all standby characters"))
                } else {
                    Ok(())
                }
            }
            Event::Heal(p, (_, c), v) => {
                f.write_fmt(format_args!("{p} {}: Healed by {v:?} HP", c.get_char_card().name))
            }
            Event::Reaction(_, _, r) => f.write_fmt(format_args!("Reaction triggered: {r:?}")),
            Event::TriggerEvent(_, e) => f.write_fmt(format_args!("Event triggered: {e:?}")),
            Event::Summon(p, s) => f.write_fmt(format_args!("{p} summoned: {}", s.get_status().name)),
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EventLog {
    pub enabled: bool,
    pub events: Vector<Event>,
}

impl std::fmt::Debug for EventLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventLog").field("enabled", &self.enabled).finish()
    }
}

impl EventLog {
    pub fn new(enabled: bool) -> EventLog {
        EventLog {
            enabled,
            events: vector![],
        }
    }

    pub fn log(&mut self, event: Event) {
        if self.enabled {
            self.events.push(event)
        }
    }

    pub fn print(&self) {
        for event in &self.events {
            let prefix = " ".repeat(2 * Into::<usize>::into(event.indent_level()));
            println!("{prefix} {:?}", event);
        }
    }

    pub fn filter<T, F: Fn(&Event) -> Option<T>>(&self, f: F) -> Vec<T> {
        let mut res = vec![];
        for e in &self.events {
            if let Some(v) = f(e) {
                res.push(v);
            }
        }
        res
    }
}
