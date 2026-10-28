use macroquad::{prelude::*, window::Conf};
use rustblocks::{
    audio::{AudioControls, AudioOutput, SoundEffect},
    game::{
        BOARD_HEIGHT, BOARD_WIDTH, Game, GameMode, Piece, PieceKind, ScoreEvent, VISIBLE_HEIGHT,
    },
    persistence::{self, PersistentData},
};
use std::{path::PathBuf, sync::atomic::Ordering};

const TITLE: &str = "RustBlocks";
const MAIN_MENU: [&str; 8] = [
    "Play",
    "Player profile",
    "Statistics",
    "Records",
    "Settings",
    "Help & controls",
    "Credits & licenses",
    "Quit",
];
const PAUSE_MENU: [&str; 5] = ["Resume", "Restart", "Quick settings", "Main menu", "Quit"];
const GAME_OVER_MENU: [&str; 3] = ["Play again", "Choose mode", "Main menu"];

fn window_conf() -> Conf {
    Conf {
        window_title: TITLE.to_owned(),
        window_width: 1040,
        window_height: 720,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Main,
    Modes,
    Playing,
    Paused,
    GameOver,
    Stats,
    Profile,
    Records,
    Settings,
    Help,
    Credits,
}

struct App {
    screen: Screen,
    settings_return: Screen,
    selection: usize,
    mode_selection: usize,
    settings_selection: usize,
    game: Option<Game>,
    data: PersistentData,
    profile_writable: bool,
    save_path: Option<PathBuf>,
    audio_controls: AudioControls,
    _audio: Option<AudioOutput>,
    move_repeat: f32,
    held_direction: i8,
    soft_drop_timer: f32,
    notification: String,
    notification_time: f32,
    profile_entry: String,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let save_path = persistence::data_path();
        let (data, profile_writable) = match save_path.as_deref().map(persistence::load) {
            Some(Ok(data)) => (data, true),
            Some(Err(error)) => {
                eprintln!("Could not load profile; defaults are in use: {error}");
                (PersistentData::default(), false)
            }
            None => (PersistentData::default(), true),
        };
        let audio_controls = AudioControls::default();
        audio_controls
            .music_enabled
            .store(data.settings.music_enabled, Ordering::Relaxed);
        audio_controls.set_volume(data.settings.music_volume * data.settings.master_volume);
        audio_controls
            .effects_enabled
            .store(data.settings.effects_enabled, Ordering::Relaxed);
        audio_controls
            .set_effects_volume(data.settings.effects_volume * data.settings.master_volume);
        let audio = match AudioOutput::start(audio_controls.clone()) {
            Ok(output) => Some(output),
            Err(error) => {
                eprintln!("Music unavailable: {error}");
                None
            }
        };
        Self {
            screen: Screen::Main,
            settings_return: Screen::Main,
            selection: 0,
            mode_selection: 0,
            settings_selection: 0,
            game: None,
            data,
            profile_writable,
            save_path,
            audio_controls,
            _audio: audio,
            move_repeat: 0.0,
            held_direction: 0,
            soft_drop_timer: 0.0,
            notification: String::new(),
            notification_time: 0.0,
            profile_entry: String::new(),
            should_quit: false,
        }
    }

    fn save(&self) {
        if self.profile_writable
            && let Some(path) = &self.save_path
            && let Err(error) = persistence::save(path, &self.data)
        {
            eprintln!("Could not save profile: {error}");
        }
    }

    fn begin_game(&mut self, mode: GameMode) {
        let seed = macroquad::miniquad::date::now().to_bits();
        self.game = Some(Game::new(mode, seed));
        self.screen = Screen::Playing;
        self.selection = 0;
        self.audio_controls.paused.store(false, Ordering::Relaxed);
        self.audio_controls.trigger(SoundEffect::Menu);
    }

    fn finish_game(&mut self) {
        let Some(game) = &self.game else { return };
        if !game.over {
            return;
        }
        self.data.games_played = self.data.games_played.saturating_add(1);
        self.data.total_lines = self
            .data
            .total_lines
            .saturating_add(u64::from(game.stats.lines));
        self.data.total_seconds = self.data.total_seconds.saturating_add(game.elapsed as u64);
        self.data.best_score = self.data.best_score.max(game.stats.score);
        let key = game.mode.name().to_owned();
        let best = self.data.best_scores.entry(key).or_default();
        *best = (*best).max(game.stats.score);
        let personal = self
            .data
            .player_scores
            .entry(self.data.active_player.clone())
            .or_default()
            .entry(game.mode.name().to_owned())
            .or_default();
        *personal = (*personal).max(game.stats.score);
        self.save();
        self.screen = Screen::GameOver;
        self.audio_controls.paused.store(true, Ordering::Relaxed);
        self.audio_controls.trigger(SoundEffect::GameOver);
    }

    fn update(&mut self, dt: f32) {
        if self.notification_time > 0.0 {
            self.notification_time -= dt;
        }
        match self.screen {
            Screen::Main => self.update_main(),
            Screen::Modes => self.update_modes(),
            Screen::Playing => self.update_game(dt),
            Screen::Paused => self.update_pause(),
            Screen::GameOver => self.update_game_over(),
            Screen::Stats | Screen::Records | Screen::Help | Screen::Credits => {
                if is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::Backspace)
                {
                    self.screen = Screen::Main;
                }
            }
            Screen::Profile => self.update_profile(),
            Screen::Settings => self.update_settings(),
        }
    }

    fn update_main(&mut self) {
        self.selection = menu_navigation(self.selection, MAIN_MENU.len());
        let mouse_clicked = menu_mouse(&MAIN_MENU, 170.0).is_some_and(|(index, clicked)| {
            self.selection = index;
            clicked
        });
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || mouse_clicked {
            match self.selection {
                0 => self.screen = Screen::Modes,
                1 => {
                    self.profile_entry = self.data.active_player.clone();
                    self.screen = Screen::Profile;
                }
                2 => self.screen = Screen::Stats,
                3 => self.screen = Screen::Records,
                4 => {
                    self.settings_return = Screen::Main;
                    self.screen = Screen::Settings;
                }
                5 => self.screen = Screen::Help,
                6 => self.screen = Screen::Credits,
                7 => self.should_quit = true,
                _ => self.should_quit = true,
            }
        }
        if is_key_pressed(KeyCode::Escape) {
            self.should_quit = true;
        }
    }

    fn update_modes(&mut self) {
        self.mode_selection = menu_navigation(self.mode_selection, GameMode::ALL.len());
        let mode_labels: Vec<_> = GameMode::ALL.iter().map(|mode| mode.name()).collect();
        let mouse_clicked = mode_mouse(&mode_labels, &mut self.mode_selection);
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || mouse_clicked {
            self.begin_game(GameMode::ALL[self.mode_selection]);
        }
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Backspace) {
            self.screen = Screen::Main;
        }
    }

    fn update_game(&mut self, dt: f32) {
        let audio = self.audio_controls.clone();
        let Some(game) = &mut self.game else { return };
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
            game.toggle_pause();
            if game.paused {
                self.screen = Screen::Paused;
                self.audio_controls.paused.store(true, Ordering::Relaxed);
            }
            return;
        }
        if (is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::X)) && game.rotate(true) {
            audio.trigger(SoundEffect::Rotate);
        }
        if is_key_pressed(KeyCode::Z) && game.rotate(false) {
            audio.trigger(SoundEffect::Rotate);
        }
        if (is_key_pressed(KeyCode::C) || is_key_pressed(KeyCode::LeftShift)) && game.hold_piece() {
            audio.trigger(SoundEffect::Menu);
        }
        if is_key_pressed(KeyCode::Space) {
            let events = game.hard_drop();
            audio.trigger(SoundEffect::Drop);
            let message = score_message(&events);
            play_score_events(&audio, &events);
            if let Some(message) = message {
                self.notification = message;
                self.notification_time = 1.2;
            }
        }
        let direction = if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            -1
        } else if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            1
        } else {
            0
        };
        if direction == 0 {
            self.held_direction = 0;
            self.move_repeat = 0.0;
        } else if direction != self.held_direction {
            self.held_direction = direction;
            self.move_repeat = -0.18;
            if game.try_move(i32::from(direction), 0) {
                audio.trigger(SoundEffect::Move);
            }
        } else {
            self.move_repeat += dt;
            if self.move_repeat >= 0.0 {
                if game.try_move(i32::from(direction), 0) {
                    audio.trigger(SoundEffect::Move);
                }
                self.move_repeat = 0.045;
            }
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            self.soft_drop_timer += dt;
            if is_key_pressed(KeyCode::Down)
                || is_key_pressed(KeyCode::S)
                || self.soft_drop_timer >= 0.045
            {
                let events = game.soft_drop();
                let message = score_message(&events);
                play_score_events(&audio, &events);
                if let Some(message) = message {
                    self.notification = message;
                    self.notification_time = 1.2;
                }
                self.soft_drop_timer = 0.0;
            }
        } else {
            self.soft_drop_timer = 0.0;
        }
        let pieces_before_tick = game.stats.pieces;
        let events = game.tick(dt);
        if game.stats.pieces > pieces_before_tick && events.is_empty() {
            audio.trigger(SoundEffect::Lock);
        }
        let message = score_message(&events);
        play_score_events(&audio, &events);
        if let Some(message) = message {
            self.notification = message;
            self.notification_time = 1.2;
        }
        if game.over {
            self.finish_game();
        }
    }

    fn update_pause(&mut self) {
        self.selection = menu_navigation(self.selection, PAUSE_MENU.len());
        let mouse_clicked = menu_mouse(&PAUSE_MENU, screen_height() / 2.0 - 45.0).is_some_and(
            |(index, clicked)| {
                self.selection = index;
                clicked
            },
        );
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
            if let Some(game) = &mut self.game {
                game.paused = false;
            }
            self.screen = Screen::Playing;
            self.audio_controls.paused.store(false, Ordering::Relaxed);
            return;
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || mouse_clicked {
            match self.selection {
                0 => {
                    if let Some(game) = &mut self.game {
                        game.paused = false;
                    }
                    self.screen = Screen::Playing;
                    self.audio_controls.paused.store(false, Ordering::Relaxed);
                }
                1 => self.begin_game(
                    self.game
                        .as_ref()
                        .map_or(GameMode::Classic, |game| game.mode),
                ),
                2 => {
                    self.settings_return = Screen::Paused;
                    self.screen = Screen::Settings;
                }
                3 => {
                    self.screen = Screen::Main;
                    self.audio_controls.paused.store(false, Ordering::Relaxed);
                }
                _ => self.should_quit = true,
            }
        }
    }

    fn update_game_over(&mut self) {
        self.selection = menu_navigation(self.selection, 3);
        let mouse_clicked = menu_mouse(&GAME_OVER_MENU, screen_height() / 2.0 - 60.0).is_some_and(
            |(index, clicked)| {
                self.selection = index;
                clicked
            },
        );
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || mouse_clicked {
            match self.selection {
                0 => self.begin_game(
                    self.game
                        .as_ref()
                        .map_or(GameMode::Classic, |game| game.mode),
                ),
                1 => self.screen = Screen::Modes,
                _ => self.screen = Screen::Main,
            }
        }
    }

    fn update_settings(&mut self) {
        let labels = 7;
        self.settings_selection = menu_navigation(self.settings_selection, labels);
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Backspace) {
            self.save();
            self.screen = self.settings_return;
            return;
        }
        if is_key_pressed(KeyCode::Left)
            || is_key_pressed(KeyCode::Right)
            || is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::Space)
        {
            let increase = is_key_pressed(KeyCode::Right)
                || is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::Space);
            match self.settings_selection {
                0 => {
                    self.data.settings.master_volume =
                        adjust(self.data.settings.master_volume, increase)
                }
                1 => {
                    self.data.settings.music_volume =
                        adjust(self.data.settings.music_volume, increase)
                }
                2 => {
                    self.data.settings.effects_volume =
                        adjust(self.data.settings.effects_volume, increase)
                }
                3 => {
                    self.data.settings.music_enabled = !self.data.settings.music_enabled;
                    self.audio_controls
                        .music_enabled
                        .store(self.data.settings.music_enabled, Ordering::Relaxed);
                }
                4 => {
                    self.data.settings.effects_enabled = !self.data.settings.effects_enabled;
                    self.audio_controls
                        .effects_enabled
                        .store(self.data.settings.effects_enabled, Ordering::Relaxed);
                }
                5 => self.data.settings.fullscreen = !self.data.settings.fullscreen,
                _ => self.data.settings.color_theme = 1 - self.data.settings.color_theme,
            }
            self.audio_controls
                .set_volume(self.data.settings.music_volume * self.data.settings.master_volume);
            self.audio_controls.set_effects_volume(
                self.data.settings.effects_volume * self.data.settings.master_volume,
            );
            if self.settings_selection == 5 {
                set_fullscreen(self.data.settings.fullscreen);
            }
            self.save();
        }
    }

    fn update_profile(&mut self) {
        while let Some(character) = get_char_pressed() {
            let letter = character.to_ascii_uppercase();
            if letter.is_ascii_alphabetic() && self.profile_entry.len() < 4 {
                self.profile_entry.push(letter);
            }
        }
        if is_key_pressed(KeyCode::Backspace) && !self.profile_entry.is_empty() {
            self.profile_entry.pop();
        }
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Main;
        } else if is_key_pressed(KeyCode::Enter) && self.profile_entry.len() == 4 {
            self.data.active_player = self.profile_entry.clone();
            self.data
                .player_scores
                .entry(self.profile_entry.clone())
                .or_default();
            self.save();
            self.screen = Screen::Main;
        }
    }

    fn draw_profile(&self) {
        draw_header();
        draw_text("PLAYER PROFILE", 0.0, 0.0, 22.0, GRAY);
        draw_text("Enter a four-letter player tag", 0.0, 230.0, 23.0, WHITE);
        draw_text(
            &self.profile_entry,
            0.0,
            292.0,
            44.0,
            Color::from_rgba(109, 226, 205, 255),
        );
        draw_text(
            "Letters only   Backspace erase   Enter save   Esc cancel",
            0.0,
            360.0,
            18.0,
            GRAY,
        );
    }

    fn draw(&self) {
        clear_background(Color::from_rgba(12, 16, 29, 255));
        match self.screen {
            Screen::Main => self.draw_main(),
            Screen::Modes => self.draw_modes(),
            Screen::Playing => if let Some(game) = &self.game { draw_game(game, &self.data, &self.notification, self.notification_time); },
            Screen::Paused => { if let Some(game) = &self.game { draw_game(game, &self.data, &self.notification, self.notification_time); } draw_overlay("PAUSED", &["Resume", "Restart", "Quick settings", "Main menu", "Quit"], self.selection); }
            Screen::GameOver => { if let Some(game) = &self.game { draw_game(game, &self.data, &self.notification, self.notification_time); } draw_game_over(&self.game, &self.data, self.selection); }
            Screen::Stats => self.draw_info("STATISTICS", &[
                format!("Games played   {}", self.data.games_played),
                format!("Lines cleared  {}", self.data.total_lines),
                format!("Best score     {}", self.data.best_score),
                format!("Time played    {} min", self.data.total_seconds / 60),
            ]),
            Screen::Profile => self.draw_profile(),
            Screen::Records => self.draw_records(),
            Screen::Settings => self.draw_settings(),
            Screen::Help => self.draw_info("CONTROLS & RULES", &[
                "Move: Left/Right or A/D  |  Soft drop: Down/S".into(),
                "Hard drop: Space  |  Rotate: Up/X clockwise, Z counter-clockwise".into(),
                "Hold: C or Left Shift  |  Pause: Escape or P".into(),
                "Complete rows to score. Gravity speeds up every 10 lines.".into(),
                "Classic and Marathon end on top-out; Sprint targets 40 lines.".into(),
                "Ultra is a two-minute score attack; Zen has slower gravity.".into(),
                "Press Escape to return.".into(),
            ]),
            Screen::Credits => self.draw_info("CREDITS & LICENSES", &[
                "RustBlocks — original interface and procedural graphics.".into(),
                "Built with Rust, macroquad, cpal, serde, TOML and directories.".into(),
                "Application code: MIT License. Third-party licenses remain with their authors.".into(),
                "No external art, recorded music, telemetry or network access.".into(),
                "Press Escape to return.".into(),
            ]),
        }
    }

    fn draw_main(&self) {
        draw_header();
        draw_center_menu(&MAIN_MENU, self.selection, 170.0);
        draw_text(
            format!("PLAYER  {}", self.data.active_player),
            24.0,
            34.0,
            18.0,
            Color::from_rgba(109, 226, 205, 255),
        );
        draw_text(
            "Navigate with ↑ ↓   Confirm with Enter   |   Rust-powered, offline and open",
            0.0,
            screen_height() - 24.0,
            16.0,
            GRAY,
        );
    }

    fn draw_modes(&self) {
        draw_header();
        draw_text("Choose how you want to play", 0.0, 0.0, 22.0, GRAY);
        let details = [
            "Endless levels and rising gravity",
            "Long-form climb to 150 lines",
            "Race to clear 40 lines",
            "Score as much as possible in 2 minutes",
            "Relaxed, slower gravity",
            "Standard rules; custom settings are limited",
        ];
        for (i, mode) in GameMode::ALL.iter().enumerate() {
            let y = 188.0 + i as f32 * 61.0;
            draw_menu_row(mode.name(), details[i], i == self.mode_selection, y);
        }
        draw_text(
            "↑ ↓ select   Enter start   Esc back",
            0.0,
            screen_height() - 32.0,
            17.0,
            GRAY,
        );
    }

    fn draw_info(&self, title: &str, lines: &[String]) {
        draw_header();
        draw_text(title, 0.0, 0.0, 22.0, GRAY);
        for (i, line) in lines.iter().enumerate() {
            draw_text(line, 0.0, 190.0 + i as f32 * 38.0, 20.0, WHITE);
        }
    }

    fn draw_records(&self) {
        draw_header();
        draw_text(
            format!("{} RECORDS BY MODE", self.data.active_player),
            0.0,
            0.0,
            22.0,
            GRAY,
        );
        let personal = self.data.player_scores.get(&self.data.active_player);
        for (i, mode) in GameMode::ALL.iter().enumerate() {
            let score = personal
                .and_then(|scores| scores.get(mode.name()))
                .copied()
                .unwrap_or(0);
            draw_text(
                format!("{:<18} {:>12}", mode.name(), score),
                0.0,
                198.0 + i as f32 * 39.0,
                22.0,
                WHITE,
            );
        }
    }

    fn draw_settings(&self) {
        draw_header();
        draw_text("SETTINGS", 0.0, 0.0, 22.0, GRAY);
        let rows = [
            format!(
                "Master volume    {:>3}%",
                (self.data.settings.master_volume * 100.0) as u32
            ),
            format!(
                "Music volume     {:>3}%",
                (self.data.settings.music_volume * 100.0) as u32
            ),
            format!(
                "Effects volume   {:>3}%",
                (self.data.settings.effects_volume * 100.0) as u32
            ),
            format!(
                "Procedural music {}",
                if self.data.settings.music_enabled {
                    "On"
                } else {
                    "Off"
                }
            ),
            format!(
                "Sound effects    {}",
                if self.data.settings.effects_enabled {
                    "On"
                } else {
                    "Off"
                }
            ),
            format!(
                "Fullscreen       {}",
                if self.data.settings.fullscreen {
                    "On"
                } else {
                    "Off"
                }
            ),
            format!(
                "Color palette    {}",
                if self.data.settings.color_theme == 0 {
                    "Vivid"
                } else {
                    "High contrast"
                }
            ),
        ];
        draw_center_menu(&rows, self.settings_selection, 200.0);
        draw_text(
            "← → change   Enter toggle   Esc save & back",
            0.0,
            screen_height() - 32.0,
            17.0,
            GRAY,
        );
    }
}

fn adjust(value: f32, up: bool) -> f32 {
    if up {
        (value + 0.05).min(1.0)
    } else {
        (value - 0.05).max(0.0)
    }
}

fn play_score_events(audio: &AudioControls, events: &[ScoreEvent]) {
    let effect = if events
        .iter()
        .any(|event| matches!(event, ScoreEvent::LevelUp(_)))
    {
        Some(SoundEffect::Level)
    } else if events
        .iter()
        .any(|event| matches!(event, ScoreEvent::Combo(_)))
    {
        Some(SoundEffect::Combo)
    } else if events.iter().any(|event| {
        matches!(
            event,
            ScoreEvent::Double | ScoreEvent::Triple | ScoreEvent::Tetris
        )
    }) {
        Some(SoundEffect::Multi)
    } else if events
        .iter()
        .any(|event| matches!(event, ScoreEvent::Single))
    {
        Some(SoundEffect::Line)
    } else {
        None
    };
    if let Some(effect) = effect {
        audio.trigger(effect);
    }
}

fn score_message(events: &[ScoreEvent]) -> Option<String> {
    if let Some(ScoreEvent::LevelUp(level)) = events
        .iter()
        .find(|event| matches!(event, ScoreEvent::LevelUp(_)))
    {
        return Some(format!("LEVEL {level}"));
    }
    if let Some(ScoreEvent::Combo(count)) = events
        .iter()
        .find(|event| matches!(event, ScoreEvent::Combo(_)))
    {
        return Some(format!("COMBO x{count}"));
    }
    events.iter().find_map(|event| match event {
        ScoreEvent::Single => Some("SINGLE".to_owned()),
        ScoreEvent::Double => Some("DOUBLE".to_owned()),
        ScoreEvent::Triple => Some("TRIPLE".to_owned()),
        ScoreEvent::Tetris => Some("FOUR LINES!".to_owned()),
        _ => None,
    })
}

fn menu_navigation(selection: usize, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    if is_key_pressed(KeyCode::Up) {
        (selection + count - 1) % count
    } else if is_key_pressed(KeyCode::Down) {
        (selection + 1) % count
    } else {
        selection
    }
}

fn menu_mouse(rows: &[impl AsRef<str>], start_y: f32) -> Option<(usize, bool)> {
    let height = (screen_height() - start_y - 40.0).max(80.0);
    let spacing = (height / rows.len().max(1) as f32).clamp(40.0, 58.0);
    let (mouse_x, mouse_y) = mouse_position();
    rows.iter().enumerate().find_map(|(index, row)| {
        let measured = measure_text(row.as_ref(), None, 26, 1.0);
        let y = start_y + index as f32 * spacing;
        let x = (screen_width() - measured.width) / 2.0;
        let inside = mouse_x >= x - 24.0
            && mouse_x <= x + measured.width + 24.0
            && mouse_y >= y - 28.0
            && mouse_y <= y + 12.0;
        inside.then_some((index, is_mouse_button_pressed(MouseButton::Left)))
    })
}

fn mode_mouse(rows: &[&str], selection: &mut usize) -> bool {
    let (mouse_x, mouse_y) = mouse_position();
    let x = screen_width() / 2.0 - 224.0;
    for (index, _) in rows.iter().enumerate() {
        let y = 188.0 + index as f32 * 61.0;
        if mouse_x >= x && mouse_x <= x + 448.0 && mouse_y >= y - 32.0 && mouse_y <= y + 20.0 {
            *selection = index;
            return is_mouse_button_pressed(MouseButton::Left);
        }
    }
    false
}

fn draw_header() {
    let title_size = (screen_width() * 0.07).clamp(34.0, 64.0);
    let title = measure_text(TITLE, None, title_size as u16, 1.0);
    draw_text(
        TITLE,
        (screen_width() - title.width) / 2.0,
        112.0,
        title_size,
        Color::from_rgba(109, 226, 205, 255),
    );
    let subtitle = measure_text("FALLING BLOCKS. YOUR RHYTHM.", None, 22, 1.0);
    draw_text(
        "FALLING BLOCKS. YOUR RHYTHM.",
        (screen_width() - subtitle.width) / 2.0,
        148.0,
        22.0,
        Color::from_rgba(148, 166, 197, 255),
    );
}

fn draw_center_menu(rows: &[impl AsRef<str>], selected: usize, start_y: f32) {
    let height = (screen_height() - start_y - 40.0).max(80.0);
    let spacing = (height / rows.len().max(1) as f32).clamp(40.0, 58.0);
    for (i, row) in rows.iter().enumerate() {
        let text = row.as_ref();
        let y = start_y + i as f32 * spacing;
        let measured = measure_text(text, None, 26, 1.0);
        let x = (screen_width() - measured.width) / 2.0;
        if i == selected {
            draw_rectangle(
                x - 24.0,
                y - 28.0,
                measured.width + 48.0,
                40.0,
                Color::from_rgba(32, 49, 70, 255),
            );
            draw_text(text, x, y, 26.0, Color::from_rgba(109, 226, 205, 255));
        } else {
            draw_text(text, x, y, 26.0, WHITE);
        }
    }
}

fn draw_menu_row(title: &str, detail: &str, selected: bool, y: f32) {
    let x = screen_width() / 2.0 - 200.0;
    if selected {
        draw_rectangle(
            x - 24.0,
            y - 32.0,
            448.0,
            52.0,
            Color::from_rgba(32, 49, 70, 255),
        );
    }
    let color = if selected {
        Color::from_rgba(109, 226, 205, 255)
    } else {
        WHITE
    };
    draw_text(title, x, y, 24.0, color);
    draw_text(
        detail,
        x + 150.0,
        y,
        17.0,
        Color::from_rgba(148, 166, 197, 255),
    );
}

fn draw_game(game: &Game, data: &PersistentData, notification: &str, notification_time: f32) {
    let cell =
        (screen_height().min(screen_width() * 0.52) / VISIBLE_HEIGHT as f32).clamp(17.0, 34.0);
    let x0 = (screen_width() - cell * 18.0) / 2.0;
    let y0 = (screen_height() - cell * VISIBLE_HEIGHT as f32) / 2.0;
    let board_x = x0 + cell * 4.0;
    let board_y = y0;
    draw_text(
        game.mode.name(),
        x0,
        y0 - 20.0,
        18.0,
        Color::from_rgba(109, 226, 205, 255),
    );
    draw_rectangle(
        board_x - 2.0,
        board_y - 2.0,
        cell * BOARD_WIDTH as f32 + 4.0,
        cell * VISIBLE_HEIGHT as f32 + 4.0,
        Color::from_rgba(42, 52, 72, 255),
    );
    draw_rectangle(
        board_x,
        board_y,
        cell * BOARD_WIDTH as f32,
        cell * VISIBLE_HEIGHT as f32,
        Color::from_rgba(17, 22, 37, 255),
    );
    for y in 0..VISIBLE_HEIGHT {
        for x in 0..BOARD_WIDTH {
            if let Some(kind) = game.board.cell(x, y + (BOARD_HEIGHT - VISIBLE_HEIGHT)) {
                draw_block(
                    board_x + x as f32 * cell,
                    board_y + y as f32 * cell,
                    cell,
                    kind,
                    data.settings.color_theme,
                );
            } else {
                draw_rectangle_lines(
                    board_x + x as f32 * cell,
                    board_y + y as f32 * cell,
                    cell,
                    cell,
                    1.0,
                    Color::from_rgba(40, 48, 65, 160),
                );
            }
        }
    }
    let mut ghost = game.active;
    while {
        let mut next = ghost;
        next.y += 1;
        if game.board.can_place(next) {
            ghost = next;
            true
        } else {
            false
        }
    } {}
    draw_piece(
        ghost,
        board_x,
        board_y,
        cell,
        data.settings.color_theme,
        true,
    );
    draw_piece(
        game.active,
        board_x,
        board_y,
        cell,
        data.settings.color_theme,
        false,
    );
    draw_text(
        format!("SCORE\n{}", game.stats.score),
        x0,
        y0 + 28.0,
        20.0,
        WHITE,
    );
    draw_text(
        format!("LEVEL\n{}", game.stats.level + 1),
        x0,
        y0 + 102.0,
        20.0,
        WHITE,
    );
    draw_text(
        format!("LINES\n{}", game.stats.lines),
        x0,
        y0 + 176.0,
        20.0,
        WHITE,
    );
    draw_text(
        format!(
            "BEST · {}\n{}",
            data.active_player,
            data.player_scores
                .get(&data.active_player)
                .and_then(|scores| scores.get(game.mode.name()))
                .copied()
                .unwrap_or(0)
        ),
        x0,
        y0 + 250.0,
        18.0,
        Color::from_rgba(148, 166, 197, 255),
    );
    let right = board_x + cell * BOARD_WIDTH as f32 + 24.0;
    draw_text("HOLD", right, y0 + 25.0, 18.0, GRAY);
    if let Some(kind) = game.hold {
        draw_preview(kind, right, y0 + 45.0, cell * 0.55);
    }
    draw_text("NEXT", right, y0 + 135.0, 18.0, GRAY);
    for (i, kind) in game.queue.iter().take(4).enumerate() {
        draw_preview(
            *kind,
            right,
            y0 + 152.0 + i as f32 * cell * 2.3,
            cell * 0.45,
        );
    }
    draw_text(
        "← → move    ↓ soft drop",
        x0,
        screen_height() - 42.0,
        16.0,
        GRAY,
    );
    draw_text(
        "Space hard drop   ↑/X/Z rotate   C hold   P pause",
        x0,
        screen_height() - 20.0,
        16.0,
        GRAY,
    );
    if notification_time > 0.0 && !notification.is_empty() {
        let text = measure_text(notification, None, 24, 1.0);
        draw_text(
            notification,
            (screen_width() - text.width) / 2.0,
            36.0,
            24.0,
            YELLOW,
        );
    }
}

fn draw_block(x: f32, y: f32, cell: f32, kind: PieceKind, palette: u8) {
    let (r, g, b) = kind.color();
    let color = if palette == 1 {
        let level = match kind {
            PieceKind::I | PieceKind::O | PieceKind::T => 220,
            _ => 130,
        };
        Color::from_rgba(level, level, level, 255)
    } else {
        Color::from_rgba(r, g, b, 255)
    };
    draw_rectangle(x + 1.0, y + 1.0, cell - 2.0, cell - 2.0, color);
    draw_rectangle(
        x + 3.0,
        y + 3.0,
        cell - 6.0,
        3.0,
        Color::from_rgba(255, 255, 255, 55),
    );
}

fn draw_piece(piece: Piece, board_x: f32, board_y: f32, cell: f32, palette: u8, ghost: bool) {
    for (x, y) in piece.cells() {
        if y >= (BOARD_HEIGHT - VISIBLE_HEIGHT) as i32 {
            let color_alpha = if ghost { 65 } else { 255 };
            let (r, g, b) = piece.kind.color();
            if ghost {
                draw_rectangle_lines(
                    board_x + x as f32 * cell + 2.0,
                    board_y + (y - (BOARD_HEIGHT - VISIBLE_HEIGHT) as i32) as f32 * cell + 2.0,
                    cell - 4.0,
                    cell - 4.0,
                    2.0,
                    Color::from_rgba(r, g, b, color_alpha),
                );
            } else {
                draw_block(
                    board_x + x as f32 * cell,
                    board_y + (y - (BOARD_HEIGHT - VISIBLE_HEIGHT) as i32) as f32 * cell,
                    cell,
                    piece.kind,
                    palette,
                );
            }
        }
    }
}

fn draw_preview(kind: PieceKind, x: f32, y: f32, cell: f32) {
    let piece = Piece {
        x: 0,
        y: 0,
        ..Piece::spawn(kind)
    };
    for (dx, dy) in piece.cells() {
        draw_block(x + dx as f32 * cell, y + dy as f32 * cell, cell, kind, 0);
    }
}

fn draw_overlay(title: &str, items: &[&str], selected: usize) {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(4, 8, 15, 195),
    );
    let size = measure_text(title, None, 48, 1.0);
    draw_text(
        title,
        (screen_width() - size.width) / 2.0,
        screen_height() / 2.0 - 115.0,
        48.0,
        WHITE,
    );
    draw_center_menu(items, selected, screen_height() / 2.0 - 45.0);
}

fn draw_game_over(game: &Option<Game>, data: &PersistentData, selected: usize) {
    let Some(game) = game else { return };
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(4, 8, 15, 205),
    );
    let completed = match game.mode {
        GameMode::Sprint => game.stats.lines >= 40,
        GameMode::Marathon => game.stats.lines >= 150,
        GameMode::Ultra => game.elapsed >= 120.0,
        _ => false,
    };
    let title = if completed {
        "MODE COMPLETE"
    } else {
        "GAME OVER"
    };
    let size = measure_text(title, None, 48, 1.0);
    draw_text(
        title,
        (screen_width() - size.width) / 2.0,
        screen_height() / 2.0 - 150.0,
        48.0,
        Color::from_rgba(235, 91, 95, 255),
    );
    draw_text(
        format!(
            "Score {}    Level {}    Lines {}    Time {}:{:02}",
            game.stats.score,
            game.stats.level + 1,
            game.stats.lines,
            game.elapsed as u64 / 60,
            game.elapsed as u64 % 60
        ),
        screen_width() / 2.0 - 190.0,
        screen_height() / 2.0 - 105.0,
        22.0,
        WHITE,
    );
    draw_text(
        format!(
            "Mode record {}",
            data.player_scores
                .get(&data.active_player)
                .and_then(|scores| scores.get(game.mode.name()))
                .copied()
                .unwrap_or(0)
        ),
        screen_width() / 2.0 - 70.0,
        screen_height() / 2.0 - 80.0,
        18.0,
        GRAY,
    );
    draw_center_menu(
        &["Play again", "Choose mode", "Main menu"],
        selected,
        screen_height() / 2.0 - 60.0,
    );
}

fn main() {
    if let Err(error) = handle_score_command() {
        eprintln!("Score sharing failed: {error}");
        std::process::exit(2);
    }
    macroquad::Window::from_config(window_conf(), async {
        let mut app = App::new();
        loop {
            let dt = get_frame_time().min(0.1);
            app.update(dt);
            if app.should_quit {
                break;
            }
            app.draw();
            next_frame().await;
        }
    });
}

fn handle_score_command() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        return Ok(());
    };
    let path = args.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Expected a file path")
    })?;
    if args.next().is_some() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Too many arguments",
        ));
    }
    let data_file = persistence::data_path()
        .ok_or_else(|| std::io::Error::other("Profile data directory unavailable"))?;
    match command.as_str() {
        "--export-scores" => {
            let data = persistence::load(&data_file)?;
            persistence::export_scores(&data, std::path::Path::new(&path))?;
            println!("Exported personal records to {path}");
        }
        "--import-scores" => {
            let mut data = persistence::load(&data_file)?;
            let count = persistence::import_scores(&mut data, std::path::Path::new(&path))?;
            persistence::save(&data_file, &data)?;
            println!("Merged {count} personal records from {path}");
        }
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unknown command",
            ));
        }
    }
    std::process::exit(0);
}
