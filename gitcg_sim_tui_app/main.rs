use crossterm::{
    event::{poll, read, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use gitcg_sim::ids::GetStatus;
use gitcg_sim::rand::{rngs::SmallRng, SeedableRng};
use gitcg_sim::smallvec::SmallVec;
use gitcg_sim::types::deal_dmg::DealDMG;
use gitcg_sim::types::logging;
use gitcg_sim::types::nondet::StandardNondetHandlerState;
use gitcg_sim_cli_utils::cli_args::{GenericSearch, SearchOpts};
use grid::{GridConstraint, GridLayout};
use std::collections::HashMap;
use std::{cmp::max, collections::VecDeque, io, time::Duration};
use structopt::StructOpt;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        canvas::{Canvas, Line},
        Block, BorderType, Borders, Cell, Paragraph, Row, Table,
    },
    Frame, Terminal,
};

use gitcg_sim::{
    action_list,
    ids::*,
    prelude::*,
    types::{card_defs::*, dice_counter::*, enums::*, game_state::*, input::*, nondet::*},
};
use gitcg_sim_search::prelude::*;

mod grid;

enum Animation {
    Message(String),
    DealDMG(DealDMG, Rect, Rect),
}

impl Animation {
    fn render<B: Backend>(&self, f: &mut Frame<B>, rect: Rect) {
        match self {
            Animation::Message(m) => {
                let v_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints(GridConstraint::to_constraints(
                        rect.height,
                        &vec![
                            GridConstraint::Fixed(1),
                            GridConstraint::Fixed(2),
                            GridConstraint::Fr(1.0),
                        ],
                    ))
                    .split(rect);

                f.render_widget(
                    Paragraph::new(format!(" {m}  ")).style(Style::default().fg(Color::White)),
                    v_chunks[1],
                )
            }
            Animation::DealDMG(_dmg, src_rect, dst_rect) => {
                let src_pos = (
                    src_rect.x + src_rect.width / 2 - rect.x,
                    rect.height - (src_rect.y + src_rect.height / 2 - rect.y),
                );
                let dst_pos = (
                    dst_rect.x + dst_rect.width / 2 - rect.x,
                    rect.height - (dst_rect.y + dst_rect.height / 2 - rect.y),
                );
                let canvas = Canvas::default()
                    .x_bounds([0.0, rect.width as f64])
                    .y_bounds([0.0, rect.height as f64])
                    .paint(|ctx| {
                        ctx.draw(&Line {
                            x1: src_pos.0 as f64,
                            y1: src_pos.1 as f64,
                            x2: dst_pos.0 as f64,
                            y2: dst_pos.1 as f64,
                            color: Color::Red,
                        });
                        //ctx.draw(&Rectangle{
                        //    x: 0.0,
                        //    y: 0.0,
                        //    width: 4.0,
                        //    height: 4.0,
                        //    color: Color::Blue,
                        //});
                        //ctx.draw(&Rectangle{
                        //    x: rect.width as f64,
                        //    y: rect.height as f64,
                        //    width: 4.0,
                        //    height: 4.0,
                        //    color: Color::Green
                        //});
                        //ctx.draw(&Rectangle{
                        //    x: (dst_pos.0 - 2) as f64,
                        //    y: (dst_pos.1 - 2) as f64,
                        //    width: 4.0,
                        //    height: 4.0,
                        //    color: Color::Yellow
                        //});
                    });
                f.render_widget(canvas, rect);
            }
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
enum RectKey {
    Character(u8),
    Summon(SummonId),
}

struct App<B: Backend> {
    pub terminal: Terminal<B>,
    pub rects: HashMap<(PlayerId, RectKey), Rect>,
    pub game: GameStateWrapper<StandardNondetHandlerState>,
    pub search: GenericSearch<StandardNondetHandlerState>,
    pub actions: SmallVec<[Input; 16]>,
    pub status: String,
    pub action_row_index: usize,
    pub scroll_y: i16,
    pub anim: VecDeque<Animation>,
}

const ANIM_SLEEP: u64 = 500;
const DEFAULT_SLEEP: u64 = 50;

impl<B: Backend> App<B> {
    fn update(&mut self) -> bool {
        let res = if let Some(winner) = self.game.winner() {
            self.status = format!("Winner: {winner:?}");
            self.actions = action_list![];
            true
        } else {
            match self.game.to_move().unwrap() {
                PlayerId::PlayerSecond => {
                    self.actions = action_list![];
                    self.status = "Opponent moving...".to_string();
                    let res = self.search.search_hidden(&self.game, PlayerId::PlayerSecond);
                    let pv = res.pv;
                    let input = pv.head().unwrap();
                    advance_and_add_logs(input, &mut self.game, &mut self.anim, &self.rects);
                    true
                }
                PlayerId::PlayerFirst => {
                    let Self { game, .. } = self;
                    let game_state = &mut game.game_state;
                    self.status = format!(
                        "  Your move.  You: (Hand: {}, Dice: {}), Opp: (Hand: {}, Dice: {}). {}",
                        game_state.players.0.hand.len(),
                        game_state.players.0.dice.total(),
                        game_state.players.1.hand.len(),
                        game_state.players.1.dice.total(),
                        {
                            match game_state.phase {
                                Phase::ActionPhase {
                                    first_end_round: Some(p),
                                    ..
                                } => {
                                    format!(
                                        " {} will move first on the next Round.",
                                        if p == PlayerId::PlayerFirst {
                                            "You"
                                        } else {
                                            "Your opponent"
                                        }
                                    )
                                }
                                _ => Default::default(),
                            }
                        }
                    );
                    self.actions = game_state.available_actions();
                    self.actions.sort_by_key(sort_key_for_action);
                    let n = self.actions.len();
                    if self.action_row_index >= n {
                        self.action_row_index = n - 1;
                    }
                    false
                }
            }
        };
        res || !self.anim.is_empty()
    }

    fn render(&mut self) -> Result<u64, io::Error> {
        let mut _animated = !self.anim.is_empty();
        let scroll_y_value = self.scroll_y;
        let Self {
            scroll_y,
            game,
            terminal,
            status,
            actions,
            action_row_index,
            anim,
            rects,
            ..
        } = self;
        let game_state = &game.game_state;
        terminal.draw(move |f| {
            let v_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(GridConstraint::to_constraints(
                    f.size().height,
                    &vec![GridConstraint::Fr(1.0), GridConstraint::Fixed(2)],
                ))
                .split(f.size());
            let ui_size = v_chunks[0];
            {
                let status_size = v_chunks[1];
                let status_bar = Paragraph::new(status.to_string());
                f.render_widget(status_bar, status_size);
            }

            let ui_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    GridConstraint::to_constraints(
                        ui_size.width,
                        &vec![
                            GridConstraint::Fixed(120),
                            GridConstraint::Fixed(70),
                            GridConstraint::Fr(1.0),
                        ],
                    )
                    .as_ref(),
                )
                .split(ui_size);

            let v_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints(GridConstraint::to_constraints(
                    ui_chunks[1].height,
                    &vec![GridConstraint::Fr(1.0), GridConstraint::Fixed(4)],
                ))
                .split(ui_chunks[1]);

            let acts_chunk = v_chunks[0];
            let dice_chunk = v_chunks[1];

            let duel_chunks = Layout::default()
                .direction(Direction::Vertical)
                .horizontal_margin(1)
                .constraints(
                    GridConstraint::to_constraints(
                        ui_chunks[0].height,
                        &vec![
                            GridConstraint::Fr(1.0),
                            GridConstraint::Fixed(1),
                            GridConstraint::Fr(1.0),
                        ],
                    )
                    .as_ref(),
                )
                .split(ui_chunks[0]);
            let info_chunk = duel_chunks[1];

            let duel_block = Block::default()
                .title("Duel")
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL);

            f.render_widget(duel_block, ui_chunks[0]);

            let info = Paragraph::new({
                let phase_part = match game_state.phase {
                    Phase::SelectStartingCharacter { .. } => "Select Starting".to_string(),
                    Phase::RollPhase { .. } => "Roll Phase".to_string(),
                    Phase::ActionPhase { first_end_round, .. } => {
                        format!("Action Phase{}", if first_end_round.is_some() { '*' } else { ' ' })
                    }
                    Phase::EndPhase { .. } => "End Phase".to_string(),
                    Phase::WinnerDecided { winner } => format!("Winner Decided: {winner}"),
                };

                let round = game_state.round_number;
                format!("Round {round} | {phase_part}")
            })
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White).bg(Color::LightBlue));
            f.render_widget(info, info_chunk);

            for player_id in [PlayerId::PlayerSecond, PlayerId::PlayerFirst] {
                let rect = duel_chunks[if player_id == PlayerId::PlayerFirst { 2 } else { 0 }];
                Self::render_player(f, game_state, player_id, rect, rects)
            }

            Self::render_actions_table(f, game_state, *action_row_index, actions, &acts_chunk, scroll_y_value);

            Self::render_dice(f, &game_state.players.0, dice_chunk);

            Self::render_log(f, game_state, ui_chunks[2], scroll_y);

            if !anim.is_empty() {
                let top = anim.pop_front().unwrap();
                top.render(f, ui_chunks[0]);
            }
        })?;

        Ok(if _animated { ANIM_SLEEP } else { DEFAULT_SLEEP })
    }

    fn render_player(
        f: &mut Frame<B>,
        game_state: &GameState,
        player_id: PlayerId,
        rect: Rect,
        rects: &mut HashMap<(PlayerId, RectKey), Rect>,
    ) {
        let player = game_state.get_player(player_id);
        let player_chunks = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(0)
            .vertical_margin(1)
            .constraints([Constraint::Percentage(90)].as_ref())
            .split(rect);

        let player_chunk = player_chunks[0];

        let n_chars = std::cmp::max(
            game_state.players.0.char_states.len(),
            game_state.players.1.char_states.len(),
        );
        let char_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints({
                let mut v = vec![GridConstraint::Fr(1.0)];
                for _ in 0..n_chars {
                    v.push(GridConstraint::Fixed(22));
                }
                v.push(GridConstraint::Fr(1.0));
                GridConstraint::to_constraints(player_chunk.width, &v)
            })
            .split(player_chunk);

        let sc = &player.status_collection;

        let support_chunks = {
            let supports_rect = char_chunks[0];
            GridLayout::default()
                .rect(supports_rect)
                .row(vec![GridConstraint::Fr(1.0), GridConstraint::Fr(1.0)])
                .col(vec![
                    GridConstraint::Fixed(3),
                    GridConstraint::Fixed(3),
                    GridConstraint::Fr(1.0),
                ])
                .split()
        };

        let mut render_box = |title, text, area| {
            let block = Block::default()
                .title(title)
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL);

            let body = Paragraph::new(text).block(block);
            f.render_widget(body, area);
        };

        for (i, ss) in sc.support_statuses_vec().iter().enumerate() {
            if i >= 4 {
                break;
            }
            let support_id = ss.support_id().unwrap();
            let support = support_id.get_status();
            render_box(
                support.name,
                format_status(&ss.state, support),
                support_chunks[i / 2][i % 2],
            );
        }

        let summon_chunks = {
            let summons_rect = char_chunks[char_chunks.len() - 1];
            GridLayout::default()
                .rect(summons_rect)
                .row(vec![GridConstraint::Fr(1.0), GridConstraint::Fr(1.0)])
                .col(vec![
                    GridConstraint::Fixed(3),
                    GridConstraint::Fixed(3),
                    GridConstraint::Fixed(3),
                    GridConstraint::Fixed(3),
                ])
                .split()
        };

        for (i, ss) in sc.summon_statuses_vec().iter().enumerate() {
            if i >= 8 {
                break;
            }
            let summon_id = ss.summon_id().unwrap();
            let summon = summon_id.get_status();
            render_box(
                summon.name,
                format_status(&ss.state, summon),
                summon_chunks[i / 2][i % 2],
            );
            rects.insert((player_id, RectKey::Summon(summon_id)), rect);
        }

        let ac = player.active_char_idx;
        for (j, c) in player.char_states.iter_all().enumerate() {
            let is_active = j == (ac as usize);
            let rect = {
                let i_first_char_chunk = 1;
                let mut rect = char_chunks[i_first_char_chunk + j];
                let dh = 1;
                if rect.height > dh {
                    rect.height -= dh;
                    if is_active {
                        if player_id == PlayerId::PlayerFirst {
                            rect.y -= dh
                        } else {
                            rect.y += dh
                        }
                    }
                }
                rect
            };
            Self::render_char_state(f, c, j, is_active, sc, &rect);
            rects.insert((player_id, RectKey::Character(j as u8)), rect);
        }
    }

    fn render_dice(f: &mut tui::Frame<B>, player: &PlayerState, rect: Rect) {
        let dice_block = Block::default()
            .title(format!("Dice ({})", player.dice.total()))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let dice_body = get_dice_body(&player.dice, player.get_element_priority()).block(dice_block);
        f.render_widget(dice_body, rect);
    }

    fn render_actions_table(
        f: &mut Frame<B>,
        gs: &GameState,
        selected_index: usize,
        acts: &[Input],
        rect: &Rect,
        _scroll_y: i16,
    ) {
        let acts_block = Block::default()
            .title("Actions")
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let rows = {
            let mut rows = vec![];
            for (i, act) in acts.iter().enumerate() {
                let k = i + 1;
                let (cost, is_fast_action) = gs.action_info(*act);
                let (a, b) = describe_action(gs.get_player(PlayerId::PlayerFirst), act);
                rows.push(
                    Row::new(vec![
                        Cell::from(format!("{k:2}")),
                        a,
                        b,
                        Cell::from(Spans::from(format_cost(cost))),
                        Cell::from(if is_fast_action { " Fast" } else { "Combat" }),
                    ])
                    .style(if i == selected_index {
                        Style::default().bg(Color::LightCyan)
                    } else {
                        Style::default()
                    }),
                );
            }
            truncate_with_scroll(&rows, selected_index as i16, rect.height - 4)
        };

        let header_row = Row::new(vec![" #", "Name", "Target", "Cost", "Action"])
            .style(Style::default().add_modifier(Modifier::BOLD));
        let widths = GridConstraint::to_constraints(
            rect.width - 7,
            &vec![
                GridConstraint::Fixed(3),
                GridConstraint::Fr(1.5),
                GridConstraint::Fr(1.0),
                GridConstraint::Fixed(7),
                GridConstraint::Fixed(7),
            ],
        );
        let acts_body = {
            Table::new(rows)
                .header(header_row)
                .block(acts_block)
                .widths(&widths)
                .column_spacing(1)
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        };

        f.render_widget(acts_body, *rect);
    }

    fn render_char_state(
        f: &mut tui::Frame<B>,
        char_state: &CharState,
        char_idx: usize,
        is_active: bool,
        sc: &StatusCollection,
        rect: &Rect,
    ) {
        let c = char_state;
        let is_dead = c.is_invalid();
        let j = char_idx;
        let char_card = c.char_id.get_char_card();
        let ci = c.char_id.get_char_card();

        let char_block = Block::default()
            .title(char_card.name.to_string())
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(if is_dead { Color::Red } else { Color::LightYellow }))
            .borders(Borders::ALL);

        if rect.width <= 2 {
            return;
        }

        let max_width = rect.width - 2;
        let body = if is_dead {
            vec![]
        } else {
            let mut lines = vec![];
            {
                let mut es = vec![Span::raw(" ")];
                for e in c.applied {
                    let col = elem_short(e).1;
                    es.push(Span::styled(format!(" {e:?} "), Style::default().bg(col)));
                    es.push(Span::raw(" "));
                }
                lines.push(Spans::from(es));
            }
            lines.push(Spans::from(vec![Span::raw(format!(
                "H {:2}/{:2}  E {:1}/{:1}",
                c.get_hp(),
                ci.max_health,
                c.get_energy(),
                ci.max_energy
            ))]));
            lines.push(Spans::from(vec![]));
            for (slot, status_id, state) in sc.equipment_statuses_vec(j as u8) {
                let status = status_id.get_status();
                let slot_part = match slot {
                    EquipSlot::Artifact => "A:",
                    EquipSlot::Weapon => "W:",
                    EquipSlot::Talent => "T:",
                };
                lines.push(Spans::from(vec![Span::styled(
                    format!(
                        "{}{}",
                        slot_part,
                        limited_concat(max_width - 2, status.name, format_status(state, status))
                    ),
                    Style::default().bg(Color::LightGreen),
                )]));
            }
            // TODO handle talent part
            if char_state.has_talent_equipped() {
                lines.push(Spans::from(vec![Span::styled(
                    "Talent",
                    Style::default().bg(Color::LightGreen),
                )]));
            }
            for s in sc.character_statuses_vec(j as u8) {
                let status = {
                    let status_id = s.status_id().unwrap();
                    status_id.get_status()
                };
                lines.push(Spans::from(vec![Span::raw(limited_concat(
                    max_width,
                    status.name,
                    format_status(&s.state, status),
                ))]));
            }
            if is_active && !sc.team_statuses_vec().is_empty() {
                lines.push(Spans::from(vec![]));
                for s in &sc.team_statuses_vec() {
                    let status = {
                        let status_id = s.status_id().unwrap();
                        status_id.get_status()
                    };
                    lines.push(Spans::from(vec![Span::raw(limited_concat(
                        max_width,
                        status.name,
                        format_status(&s.state, status),
                    ))]));
                }
            }

            lines
        };
        let char_body = Paragraph::new(body)
            .block(char_block)
            .style(Style::default().fg(Color::White));
        f.render_widget(char_body, *rect);

        if is_dead {
            let canvas = Canvas::default()
                .block(Block::default().title("").border_type(BorderType::Rounded))
                .x_bounds([0.0, 1.0])
                .y_bounds([0.0, 1.0])
                .paint(|ctx| {
                    ctx.draw(&Line {
                        x1: 0.0,
                        y1: 0.0,
                        x2: 1.0,
                        y2: 1.0,
                        color: Color::Red,
                    });
                    ctx.draw(&Line {
                        x1: 1.0,
                        y1: 0.0,
                        x2: 0.0,
                        y2: 1.0,
                        color: Color::Red,
                    })
                });
            let mut draw_rect = *rect;
            draw_rect.height -= 1;
            draw_rect.width -= 2;
            draw_rect.x += 1;
            f.render_widget(canvas, draw_rect);
        }
    }

    fn render_log(f: &mut Frame<B>, game_state: &GameState, rect: Rect, scroll_y: &i16) {
        let log_block = Block::default()
            .title("Log")
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);
        let log_lines = {
            let mut log_lines = Vec::with_capacity(game_state.log.events.len() + 4);
            let mut i = 0;
            for e in &game_state.log.events {
                if e.indent_level() < 4 {
                    let prefix = " ".repeat(2 * Into::<usize>::into(e.indent_level()));
                    let line = format!("{e}");
                    if !line.is_empty() {
                        log_lines.push(format!("{i:3} {prefix}{line}"));
                        i += 1;
                    }
                }
            }
            log_lines.push(String::default());
            let log_lines = truncate_with_scroll(&log_lines, (log_lines.len() as i16) + *scroll_y + 2, rect.height);
            log_lines.join("\n")
        };
        let log_body = Paragraph::new(log_lines).block(log_block);
        f.render_widget(log_body, rect);
    }

    fn keyboard(&mut self, skip_keys: bool) -> crossterm::Result<bool> {
        // let winner_found = self.game.to_move().is_some();
        if skip_keys {
            if let Event::Key(kc) = read()? {
                if let KeyCode::Char('q') = kc.code {
                    return Ok(true);
                }
            }
        } else if let Event::Key(kc) = read()? {
            if self.game.to_move() == Some(PlayerId::PlayerFirst) {
                let mut input = None;
                match kc.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                        let act_idx = (c as usize) - ('1' as usize);
                        let acts = &self.actions;
                        if act_idx < acts.len() {
                            input = Some(acts[act_idx]);
                        }
                    }
                    KeyCode::Char('k') => {
                        if self.action_row_index == 0 {
                            self.action_row_index = self.actions.len() - 1
                        } else {
                            self.action_row_index -= 1
                        }
                    }
                    KeyCode::Char('j') => {
                        if self.action_row_index == self.actions.len() - 1 {
                            self.action_row_index = 0
                        } else {
                            self.action_row_index += 1
                        }
                    }
                    KeyCode::Char(_) => {}
                    KeyCode::Down => {
                        self.scroll_y += 1;
                    }
                    KeyCode::Up => {
                        self.scroll_y -= 1;
                    }
                    KeyCode::PageDown => {
                        self.scroll_y += 8;
                    }
                    KeyCode::PageUp => {
                        self.scroll_y -= 8;
                    }
                    KeyCode::Enter => {
                        let acts = &self.actions;
                        input = Some(acts[self.action_row_index]);
                    }
                    _ => (),
                }

                if let Some(input) = input {
                    advance_and_add_logs(input, &mut self.game, &mut self.anim, &self.rects);
                }
            }
        }
        Ok(false)
    }
}

fn advance_and_add_logs<S: std::fmt::Debug + NondetState>(
    input: Input,
    game: &mut GameStateWrapper<S>,
    anim: &mut VecDeque<Animation>,
    rects: &HashMap<(PlayerId, RectKey), Rect>,
) {
    let log_idx = game.game_state.log.events.len();
    game.advance(input).unwrap();
    add_logs(log_idx, &game.game_state, anim, rects);
}

fn add_logs(
    log_idx: usize,
    game_state: &GameState,
    anim: &mut VecDeque<Animation>,
    rects: &HashMap<(PlayerId, RectKey), Rect>,
) {
    let new_log_idx = game_state.log.events.len();
    let new_log_entries = game_state.log.events[log_idx..new_log_idx].to_vec();
    for entry in new_log_entries {
        if let logging::Event::DealDMG(_src, (dst_player_id, (dst_char_idx, _)), deal_dmg) = entry {
            let src_player_id = dst_player_id.opposite();
            let src_char_idx = game_state.get_player(src_player_id).active_char_idx;
            let Some(src) = rects.get(&(src_player_id, RectKey::Character(src_char_idx))) else {
                continue;
            };
            let Some(dst) = rects.get(&(dst_player_id, RectKey::Character(dst_char_idx))) else {
                continue;
            };
            anim.push_back(Animation::DealDMG(deal_dmg, *src, *dst));
            continue;
        }
        if entry.indent_level() <= 3 {
            anim.push_back(Animation::Message(format!("{entry}")))
        }
    }
}

fn truncate_with_scroll<T: Clone>(rows: &[T], scroll_y: i16, height: u16) -> Vec<T> {
    rows.iter()
        .skip(max(0, scroll_y - (height as i16)) as usize)
        .cloned()
        .collect()
}

fn format_status(es: &AppliedEffectState, s: &'static Status) -> String {
    let part1 = if s.usages_as_shield_points {
        format!("<{}>", es.get_usages())
    } else if s.usages.is_some() {
        format!("[{}]", es.get_usages())
    } else if s.duration_rounds.is_some() {
        format!("({})", es.get_duration())
    } else {
        String::default()
    };

    if s.counter_spec.is_some() {
        format!("{part1}({})", es.get_counter())
    } else {
        part1
    }
}

fn format_cost<'a>(cost: Cost) -> Vec<Span<'a>> {
    let mut parts = vec![];
    if let Some((e, v)) = cost.elem_cost {
        let (e, col) = elem_short(e);
        parts.push(Span::styled(
            format!("{e}{v}"),
            Style::default().fg(Color::White).bg(col).add_modifier(Modifier::BOLD),
        ))
    }
    if cost.aligned_cost > 0 {
        if !parts.is_empty() {
            parts.push(Span::raw(" "));
        }
        parts.push(Span::styled(
            format!("M{}", cost.aligned_cost),
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
    }
    if cost.unaligned_cost > 0 {
        if !parts.is_empty() {
            parts.push(Span::raw(" "));
        }
        parts.push(Span::styled(
            format!("{}", cost.unaligned_cost),
            Style::default().fg(Color::White).bg(Color::DarkGray),
        ))
    }
    if cost.energy_cost > 0 {
        if !parts.is_empty() {
            parts.push(Span::raw("+"));
        }
        parts.push(Span::styled(
            format!("E{}", cost.energy_cost),
            Style::default().fg(Color::White).bg(Color::Yellow),
        ))
    }
    parts
}

fn limited_concat(max_width: u16, left: &'static str, right: String) -> String {
    let max_width = max_width as usize;
    if left.len() + right.len() + 1 < max_width {
        format!("{} {}", left, right)
    } else {
        let len = if right.len() + 2 >= max_width {
            0
        } else {
            max_width - right.len() - 2
        };
        format!("{}\u{2026} {}", &left[0..len], right)
    }
}

fn get_dice_body(dice: &DiceCounter, ep: ElementPriority) -> Paragraph {
    let mut lines = Vec::with_capacity(2);
    let mut v = Vec::with_capacity(8);
    let dice_list = {
        let mut v = Vec::with_capacity(dice.total() as usize);
        for (d, c) in dice.tally() {
            for _ in 0..c {
                v.push(d)
            }
        }
        v.sort_by_key(|&k| match k {
            Dice::Omni => 0u8,
            Dice::Elem(e) if ep.important_elems.contains(e) => 1,
            _ => 2,
        });
        v
    };
    let mut i = 0;
    for d in dice_list {
        i += 1;
        if i % 14 == 0 {
            lines.push(Spans::from(v.clone()));
            v.clear();
            v.push(Span::raw(" "))
        } else {
            v.push(Span::raw(" "));
        }
        match d {
            Dice::Omni => v.push(Span::styled(" O ", Style::default().fg(Color::Black).bg(Color::White))),
            Dice::Elem(e) => {
                let (name, col) = elem_short(e);
                v.push(Span::styled(
                    format!(" {name} "),
                    Style::default().fg(Color::White).bg(col),
                ));
            }
        }
    }
    if !v.is_empty() {
        lines.push(Spans::from(v.clone()));
    }
    Paragraph::new(lines)
}

fn elem_short(e: Element) -> (&'static str, Color) {
    match e {
        Element::Pyro => ("P", Color::Red),
        Element::Hydro => ("H", Color::Blue),
        Element::Cryo => ("C", Color::LightBlue),
        Element::Electro => ("E", Color::Magenta),
        Element::Dendro => ("D", Color::Green),
        Element::Geo => ("G", Color::Yellow),
        Element::Anemo => ("A", Color::Cyan),
    }
}

fn sort_key_for_action(input: &Input) -> String {
    match *input {
        Input::NoAction => String::default(),
        Input::NondetResult(..) => String::default(),
        Input::FromPlayer(_, action) => match action {
            PlayerAction::EndRound => "9".to_string(),
            PlayerAction::ElementalTuning(card_id) => format!("8{}", card_id.get_card().name),
            PlayerAction::PlayCard(card_id, tgt) => {
                format!("7{} {:?}", card_id.get_card().name, tgt)
            }
            PlayerAction::SwitchCharacter(i) => format!("6{i}"),
            PlayerAction::CastSkill(skill_id) => {
                let skill = skill_id.get_skill();
                format!(
                    "5{}",
                    match skill.skill_type {
                        SkillType::NormalAttack => 0,
                        SkillType::ElementalSkill => 1,
                        SkillType::ElementalBurst => 2,
                    }
                )
            }
            PlayerAction::PostDeathSwitch(i) => format!("0{i}"),
        },
    }
}

fn describe_action<'a, 'b>(player_state: &'b PlayerState, input: &'b Input) -> (Cell<'a>, Cell<'a>) {
    let get_character_name = |i: u8| -> Cell<'a> {
        Cell::from(player_state.char_states[i].char_id.get_char_card().name).style(Style::default().fg(Color::Yellow))
    };

    fn card_cell<'a>(card_id: CardId) -> Cell<'a> {
        Cell::from(card_id.get_card().name).style(Style::default().fg(Color::LightGreen))
    }

    fn et_cell<'a>(card_id: CardId) -> Cell<'a> {
        Cell::from(Spans::from(vec![
            Span::from("ET: "),
            Span::styled(card_id.get_card().name, Style::default().fg(Color::LightGreen)),
        ]))
    }

    fn skill_cell<'a>(skill_id: SkillId) -> Cell<'a> {
        Cell::from(skill_id.get_skill().name).style(Style::default().fg(Color::LightBlue))
    }

    macro_rules! empty {
        () => {
            Cell::from("")
        };
    }

    match *input {
        Input::NoAction => (empty!(), empty!()),
        Input::NondetResult(_) => (empty!(), empty!()),
        Input::FromPlayer(_, action) => match action {
            PlayerAction::EndRound => (Cell::from("End Round").style(Style::default().fg(Color::Red)), empty!()),
            PlayerAction::PlayCard(card_id, tgt) => {
                let tgt_desc = match tgt {
                    None => empty!(),
                    Some(CardSelection::OwnCharacter(i)) => get_character_name(i),
                    Some(CardSelection::OwnSummon(summon_id)) => format!("Own:{}", summon_id.get_status().name).into(),
                    Some(CardSelection::OpponentSummon(summon_id)) => {
                        format!("Opp:{}", summon_id.get_status().name).into()
                    }
                };
                (card_cell(card_id), tgt_desc)
            }
            PlayerAction::ElementalTuning(card_id) => (et_cell(card_id), empty!()),
            PlayerAction::CastSkill(skill_id) => (skill_cell(skill_id), empty!()),
            PlayerAction::SwitchCharacter(i) | PlayerAction::PostDeathSwitch(i) => {
                (Cell::from("Switch: "), get_character_name(i))
            }
        },
    }
}

pub fn main() -> Result<(), io::Error> {
    let mut deck_opts = SearchOpts::from_args();
    // Ignore debug flag
    deck_opts.search.debug = false;
    let (decklist1, decklist2) = deck_opts.get_decks()?;
    {
        let rand1 = SmallRng::seed_from_u64(deck_opts.seed.unwrap_or(100));
        let mut game = new_standard_game(&decklist1, &decklist2, rand1);
        if deck_opts.tactical {
            game.convert_to_tactical_search();
        }
        let search = deck_opts.make_search(true, deck_opts.get_limits());

        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            //EnableMouseCapture
        )?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let mut app_state = App {
            terminal,
            game,
            search,
            status: "".to_string(),
            actions: action_list![],
            action_row_index: 0,
            scroll_y: 0,
            anim: Default::default(),
            rects: Default::default(),
        };

        loop {
            let skip_keys = app_state.update();
            let sleep = app_state.render()?;
            let dur = Duration::from_millis(sleep);
            if !poll(dur)? {
                continue;
            }
            if app_state.keyboard(skip_keys)? {
                break;
            }
        }

        // restore terminal
        disable_raw_mode()?;
        execute!(
            app_state.terminal.backend_mut(),
            LeaveAlternateScreen,
            //DisableMouseCapture
        )?;
        app_state.terminal.show_cursor()?;
    }

    Ok(())
}
