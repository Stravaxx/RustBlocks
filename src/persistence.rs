use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

const PROFILE_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub effects_volume: f32,
    pub music_enabled: bool,
    pub effects_enabled: bool,
    pub fullscreen: bool,
    pub color_theme: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            music_volume: 0.18,
            effects_volume: 0.7,
            music_enabled: true,
            effects_enabled: true,
            fullscreen: false,
            color_theme: 0,
        }
    }
}

impl Settings {
    pub fn validate(&mut self) {
        self.master_volume = finite_volume(self.master_volume, 0.8);
        self.music_volume = finite_volume(self.music_volume, 0.18);
        self.effects_volume = finite_volume(self.effects_volume, 0.7);
        self.color_theme = self.color_theme.min(1);
    }
}

fn finite_volume(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        fallback
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistentData {
    pub schema_version: u32,
    pub settings: Settings,
    pub best_scores: std::collections::BTreeMap<String, u64>,
    pub games_played: u64,
    pub total_lines: u64,
    pub best_score: u64,
    pub total_seconds: u64,
    /// Four-letter local player tag used for per-player records.
    pub active_player: String,
    /// Personal records, keyed by the player's four-letter tag.
    pub player_scores: std::collections::BTreeMap<String, std::collections::BTreeMap<String, u64>>,
}

impl Default for PersistentData {
    fn default() -> Self {
        Self {
            schema_version: PROFILE_VERSION,
            settings: Settings::default(),
            best_scores: std::collections::BTreeMap::new(),
            games_played: 0,
            total_lines: 0,
            best_score: 0,
            total_seconds: 0,
            active_player: "PLAY".to_owned(),
            player_scores: std::collections::BTreeMap::new(),
        }
    }
}

pub fn data_path() -> Option<PathBuf> {
    ProjectDirs::from("org", "RustBlocks", "RustBlocks")
        .map(|dirs| dirs.data_dir().join("profile.toml"))
}

pub fn load(path: &Path) -> io::Result<PersistentData> {
    if let Ok(metadata) = fs::metadata(path)
        && metadata.len() > 1_048_576
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Profile file exceeds the 1 MiB size limit",
        ));
    }
    match fs::read_to_string(path) {
        Ok(text) => {
            let mut data: PersistentData = toml::from_str(&text)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            if data.schema_version > PROFILE_VERSION {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Profile was written by a newer RustBlocks version",
                ));
            }
            data.settings.validate();
            Ok(data)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(PersistentData::default()),
        Err(error) => Err(error),
    }
}

pub fn save(path: &Path, data: &PersistentData) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Profile path has no parent directory",
        )
    })?;
    fs::create_dir_all(parent)?;
    let content = toml::to_string_pretty(data).map_err(io::Error::other)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(content.as_bytes())?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(path)
        .map_err(|error| io::Error::other(error.error))?;
    Ok(())
}

/// Export personal bests as a portable, headered CSV file suitable for USB sharing.
pub fn export_scores(data: &PersistentData, path: &Path) -> io::Result<()> {
    let mut output = String::from("player,mode,best_score\n");
    for (player, scores) in &data.player_scores {
        if !valid_player_tag(player) {
            continue;
        }
        for (mode, score) in scores {
            output.push_str(&format!("{player},{mode},{score}\n"));
        }
    }
    fs::write(path, output)
}

/// Merge a shared score CSV into the local profile, keeping the best score per mode.
pub fn import_scores(data: &mut PersistentData, path: &Path) -> io::Result<usize> {
    let contents = fs::read_to_string(path)?;
    let mut lines = contents.lines();
    if lines.next() != Some("player,mode,best_score") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid score CSV header",
        ));
    }
    let mut count = 0;
    for (index, line) in lines.enumerate() {
        let fields: Vec<_> = line.split(',').collect();
        if fields.len() != 3 || !valid_player_tag(fields[0]) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid score row {}", index + 2),
            ));
        }
        let known_mode = crate::game::GameMode::ALL
            .iter()
            .any(|mode| mode.name() == fields[1]);
        let score = fields[2].parse::<u64>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid score row {}", index + 2),
            )
        })?;
        if !known_mode {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown game mode on row {}", index + 2),
            ));
        }
        let best = data
            .player_scores
            .entry(fields[0].to_owned())
            .or_default()
            .entry(fields[1].to_owned())
            .or_default();
        *best = (*best).max(score);
        count += 1;
    }
    Ok(count)
}

fn valid_player_tag(tag: &str) -> bool {
    tag.len() == 4 && tag.bytes().all(|byte| byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_file_is_reported_without_replacing_it() {
        let path =
            std::env::temp_dir().join(format!("rustblocks-profile-{}.toml", std::process::id()));
        fs::write(&path, "not = [valid").unwrap();
        assert_eq!(load(&path).unwrap_err().kind(), io::ErrorKind::InvalidData);
        assert_eq!(fs::read_to_string(&path).unwrap(), "not = [valid");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn refuses_oversized_profile_files() {
        let path = std::env::temp_dir().join(format!(
            "rustblocks-large-profile-{}.toml",
            std::process::id()
        ));
        fs::write(&path, vec![b' '; 1_048_577]).unwrap();
        assert_eq!(load(&path).unwrap_err().kind(), io::ErrorKind::InvalidData);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn refuses_profiles_from_a_newer_schema_without_modifying_them() {
        let path = std::env::temp_dir().join(format!(
            "rustblocks-new-profile-{}.toml",
            std::process::id()
        ));
        fs::write(&path, "schema_version = 999").unwrap();
        assert_eq!(load(&path).unwrap_err().kind(), io::ErrorKind::InvalidData);
        assert_eq!(fs::read_to_string(&path).unwrap(), "schema_version = 999");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn saves_and_loads_profile() {
        let path =
            std::env::temp_dir().join(format!("rustblocks-profile-{}.toml", std::process::id()));
        let data = PersistentData {
            best_score: 123,
            ..PersistentData::default()
        };
        save(&path, &data).unwrap();
        assert_eq!(load(&path).unwrap().best_score, 123);
        let replacement = PersistentData {
            best_score: 456,
            ..PersistentData::default()
        };
        save(&path, &replacement).unwrap();
        assert_eq!(load(&path).unwrap().best_score, 456);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn score_exchange_merges_best_per_player_and_mode() {
        let input =
            std::env::temp_dir().join(format!("rustblocks-scores-{}.csv", std::process::id()));
        let data = PersistentData::default();
        export_scores(&data, &input).unwrap();
        fs::write(
            &input,
            "player,mode,best_score\nALFA,Classic,120\nBETA,Sprint 40,80\nALFA,Classic,90\n",
        )
        .unwrap();
        let mut imported = PersistentData::default();
        assert_eq!(import_scores(&mut imported, &input).unwrap(), 3);
        assert_eq!(imported.player_scores["ALFA"]["Classic"], 120);
        assert_eq!(imported.player_scores["BETA"]["Sprint 40"], 80);
        fs::write(&input, "player,mode,best_score\nALFA,Classic,140\n").unwrap();
        import_scores(&mut imported, &input).unwrap();
        assert_eq!(imported.player_scores["ALFA"]["Classic"], 140);
        fs::remove_file(input).unwrap();
    }

    #[test]
    fn score_import_rejects_invalid_tags_and_modes() {
        let input = std::env::temp_dir().join(format!(
            "rustblocks-invalid-scores-{}.csv",
            std::process::id()
        ));
        let mut data = PersistentData::default();
        fs::write(&input, "player,mode,best_score\nTOOLONG,Classic,1\n").unwrap();
        assert_eq!(
            import_scores(&mut data, &input).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        fs::write(&input, "player,mode,best_score\nALFA,Unknown,1\n").unwrap();
        assert_eq!(
            import_scores(&mut data, &input).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        fs::remove_file(input).unwrap();
    }
}
