use super::*;

pub const C: Card = Card {
    name: "I Haven't Lost Yet!",
    cost: Cost::ZERO,
    card_type: CardType::Event,
    card_impl: Some(&I),
    effects: list8![
        Command::AddDice(DiceCounter::omni(1)),
        Command::AddEnergy(1, CmdCharIdx::Active),
    ],
};

pub struct IHaventLostYet();

pub const I: IHaventLostYet = IHaventLostYet();

impl CardImpl for IHaventLostYet {
    fn can_be_played(&self, cic: &CardImplContext) -> CanBePlayedResult {
        let player = cic.game_state.get_player(cic.active_player_id);
        // TODO apply a status to enforce once per turn
        if player.flags.contains(PlayerFlag::DiedThisRound) {
            CanBePlayedResult::CanBePlayed
        } else {
            CanBePlayedResult::CannotBePlayed
        }
    }
}
