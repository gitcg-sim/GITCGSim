use super::*;

#[derive(Default, Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DMGInfo {
    pub target_hp: u8,
    pub target_affected_by_riptide: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CommandSource {
    /// The command is dispatched by game mechanics.
    #[default]
    Event,
    Card {
        card_id: CardId,
        target: Option<CardSelection>,
    },
    Switch {
        from_char_idx: u8,
        dst_char_idx: u8,
    },
    Skill {
        char_idx: u8,
        skill_id: SkillId,
    },
    Character {
        char_idx: u8,
    },
}

impl CommandSource {
    /// Matches `Skill`
    #[inline]
    pub fn skill_id(&self) -> Option<SkillId> {
        if let CommandSource::Skill { skill_id, .. } = self {
            Some(*skill_id)
        } else {
            None
        }
    }

    /// Matches `Switch`
    #[inline]
    pub fn dst_char_idx(&self) -> Option<u8> {
        if let CommandSource::Switch { dst_char_idx, .. } = self {
            Some(*dst_char_idx)
        } else {
            None
        }
    }

    /// Matches `Skill`
    #[inline]
    pub fn char_idx(&self) -> Option<u8> {
        match self {
            CommandSource::Skill { char_idx, .. } => Some(*char_idx),
            CommandSource::Character { char_idx, .. } => Some(*char_idx),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandTarget {
    pub player_id: PlayerId,
    pub char_idx: u8,
}

impl CommandTarget {
    #[inline]
    pub fn new(player_id: PlayerId, char_idx: u8) -> Self {
        Self { player_id, char_idx }
    }
}

/// A `CommandContext` contains information about the source and targeting of the command.
/// A command is always performed by a player (`src_player_id`) with a specific source (`src`)
/// under the source player's context. The target (`tgt`) is always defined under the opponent
/// of the source player's context.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandContext {
    pub src_player_id: PlayerId,
    pub src: CommandSource,
    pub tgt: Option<CommandTarget>,
}

impl CommandContext {
    pub const EMPTY: Self = Self {
        src_player_id: PlayerId::PlayerFirst,
        src: CommandSource::Event,
        tgt: None,
    };

    #[inline]
    pub fn new(src_player_id: PlayerId, src: CommandSource, tgt: Option<CommandTarget>) -> Self {
        Self {
            src_player_id,
            src,
            tgt,
        }
    }

    #[inline]
    pub fn new_event(src_player_id: PlayerId) -> Self {
        Self {
            src_player_id,
            src: CommandSource::Event,
            tgt: None,
        }
    }

    #[inline]
    pub fn without_target(&self) -> Self {
        Self { tgt: None, ..*self }
    }

    #[inline]
    pub fn with_src(&self, src: CommandSource) -> Self {
        Self { src, ..*self }
    }

    #[inline]
    pub fn with_tgt(&self, tgt: Option<CommandTarget>) -> Self {
        Self { tgt, ..*self }
    }

    #[inline]
    pub fn with_character_index(&self, char_idx: u8) -> Self {
        let tgt = Some(CommandTarget::new(self.src_player_id.opposite(), char_idx));
        Self { tgt, ..*self }
    }
}
