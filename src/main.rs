use eframe::egui;
use egui::{Color32, RichText};
use rodio::{Decoder, OutputStream, Sink};
use rfd::FileDialog;
use std::collections::{HashMap, VecDeque};
use std::fs::{self, File};
use std::io::{BufReader, copy};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Clone, Debug)]
pub struct Track {
    pub path: PathBuf,
    pub title: String,
    pub duration: Option<Duration>,
}

impl Track {
    pub fn new(path: PathBuf) -> Self {
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        let duration = Self::get_duration(&path).ok();
        
        Self {
            path,
            title,
            duration,
        }
    }
    
    fn get_duration(path: &Path) -> Result<Duration, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        
        let mut hint = Hint::new();
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                hint.with_extension(ext_str);
            }
        }
        
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();
        
        let probed = symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts)?;
        let format = probed.format;
        
        if let Some(track) = format.tracks().first() {
            if let Some(time_base) = track.codec_params.time_base {
                if let Some(n_frames) = track.codec_params.n_frames {
                    let duration_secs = (n_frames as f64 * time_base.numer as f64) / time_base.denom as f64;
                    return Ok(Duration::from_secs_f64(duration_secs));
                }
            }
        }
        
        // Если не удалось получить точную длительность, возвращаем примерную
        Ok(Duration::from_secs(180))
    }
}

#[derive(Clone, Debug)]
pub struct Playlist {
    pub name: String,
    pub tracks: VecDeque<Track>,
    pub folder_path: PathBuf,
}

impl Playlist {
    pub fn new(name: String, music_folder: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let folder_path = music_folder.join(&name);
        if !folder_path.exists() {
            fs::create_dir_all(&folder_path)?;
        }
        
        Ok(Playlist {
            name,
            tracks: VecDeque::new(),
            folder_path,
        })
    }
    
    pub fn load_tracks(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.tracks.clear();
        
        if self.folder_path.exists() {
            for entry in fs::read_dir(&self.folder_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && Self::is_audio_file(&path) {
                    let track = Track::new(path);
                    self.tracks.push_back(track);
                }
            }
        }
        
        Ok(())
    }
    
    fn is_audio_file(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(ext.to_str(), Some("mp3") | Some("wav") | Some("flac") | Some("ogg"))
        } else {
            false
        }
    }
    
    pub fn add_track_from_path(&mut self, source_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if !Self::is_audio_file(source_path) {
            return Err("Unsupported audio format".into());
        }
        
        let filename = source_path.file_name()
            .ok_or("Invalid file name")?;
        let destination = self.folder_path.join(filename);
        
        // Копируем файл в папку плейлиста
        let mut source_file = File::open(source_path)?;
        let mut dest_file = File::create(&destination)?;
        copy(&mut source_file, &mut dest_file)?;
        
        // Добавляем трек в плейлист
        let track = Track::new(destination);
        self.tracks.push_back(track);
        
        Ok(())
    }
    
    pub fn remove_track(&mut self, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        if index < self.tracks.len() {
            if let Some(track) = self.tracks.remove(index) {
                // Удаляем файл из файловой системы
                if track.path.exists() {
                    fs::remove_file(&track.path)?;
                }
            }
        }
        Ok(())
    }
    
    pub fn clear_tracks(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Удаляем все файлы из папки плейлиста
        if self.folder_path.exists() {
            for entry in fs::read_dir(&self.folder_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && Self::is_audio_file(&path) {
                    fs::remove_file(&path)?;
                }
            }
        }
        self.tracks.clear();
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

pub struct AudioPlayer {
    sink: Option<Sink>,
    _stream: Option<OutputStream>,
    state: PlaybackState,
    current_track: Option<Track>,
    playlists: HashMap<String, Playlist>,
    current_playlist: Option<String>,
    current_index: usize,
    volume: f32,
    position: Duration,
    last_update: Instant,
    show_file_dialog: bool,
    show_playlist_dialog: bool,
    show_add_track_dialog: bool,
    new_playlist_name: String,
    music_folder: PathBuf,
}

impl AudioPlayer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let music_folder = PathBuf::from("./music");
        // Создаем папку music если её нет
        if !music_folder.exists() {
            fs::create_dir(&music_folder)?;
        }

        // Создаем плейлист по умолчанию
        let mut playlists = HashMap::new();
        let default_playlist = Playlist::new("Default".to_string(), &music_folder)?;
        playlists.insert("Default".to_string(), default_playlist);

        Ok(AudioPlayer {
            sink: None,
            _stream: None,
            state: PlaybackState::Stopped,
            current_track: None,
            playlists,
            current_playlist: Some("Default".to_string()),
            current_index: 0,
            volume: 0.5,
            position: Duration::from_secs(0),
            last_update: Instant::now(),
            show_file_dialog: false,
            show_playlist_dialog: false,
            show_add_track_dialog: false,
            new_playlist_name: String::new(),
            music_folder,
        })
    }

    pub fn create_playlist(&mut self, name: String) -> Result<(), Box<dyn std::error::Error>> {
        if !self.playlists.contains_key(&name) {
            let playlist = Playlist::new(name.clone(), &self.music_folder)?;
            self.playlists.insert(name.clone(), playlist);
            self.current_playlist = Some(name);
        }
        Ok(())
    }

    pub fn load_playlists(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.playlists.clear();
        
        // Создаем плейлист по умолчанию
        let default_playlist = Playlist::new("Default".to_string(), &self.music_folder)?;
        self.playlists.insert("Default".to_string(), default_playlist);
        
        if self.music_folder.exists() {
            for entry in fs::read_dir(&self.music_folder)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name != "Default" {
                            let mut playlist = Playlist::new(name.to_string(), &self.music_folder)?;
                            playlist.load_tracks()?;
                            self.playlists.insert(name.to_string(), playlist);
                        }
                    }
                }
            }
            
            // Загружаем треки для плейлиста по умолчанию (файлы в корне music/)
            if let Some(default_playlist) = self.playlists.get_mut("Default") {
                for entry in fs::read_dir(&self.music_folder)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() && Playlist::is_audio_file(&path) {
                        let track = Track::new(path);
                        default_playlist.tracks.push_back(track);
                    }
                }
            }
        }
        
        if self.current_playlist.is_none() || !self.playlists.contains_key(self.current_playlist.as_ref().unwrap()) {
            self.current_playlist = Some("Default".to_string());
        }
        
        self.current_index = 0;
        Ok(())
    }

    pub fn add_track_to_playlist(&mut self, source_path: &Path, playlist_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(playlist) = self.playlists.get_mut(playlist_name) {
            playlist.add_track_from_path(source_path)?;
        } else {
            return Err("Playlist not found".into());
        }
        Ok(())
    }

    pub fn switch_playlist(&mut self, playlist_name: String) {
        if self.playlists.contains_key(&playlist_name) {
            self.current_playlist = Some(playlist_name);
            self.current_index = 0;
            self.stop();
        }
    }

    pub fn select_and_add_tracks(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let current_playlist_name = self.current_playlist.clone();
        if let Some(current_playlist_name) = current_playlist_name {
            let files = FileDialog::new()
                .add_filter("Audio Files", &["mp3", "wav", "flac", "ogg"])
                .set_title("Select Audio Files")
                .pick_files();

            if let Some(files) = files {
                for file_path in files {
                    if let Err(e) = self.add_track_to_playlist(&file_path, &current_playlist_name) {
                        eprintln!("Failed to add track: {}", e);
                    }
                }
            }
        }
        Ok(())
    }
    
    pub fn remove_track_from_current_playlist(&mut self, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(current_playlist_name) = self.current_playlist.clone() {
            let tracks_len = if let Some(playlist) = self.playlists.get(&current_playlist_name) {
                playlist.tracks.len()
            } else {
                return Ok(());
            };
            
            if let Some(playlist) = self.playlists.get_mut(&current_playlist_name) {
                playlist.remove_track(index)?;
            }
            
            // Если удаленный трек был текущим, останавливаем воспроизведение
            if index == self.current_index {
                self.stop();
                self.current_track = None;
            } else if index < self.current_index {
                // Если удаленный трек был до текущего, сдвигаем индекс
                self.current_index -= 1;
            }
            
            // Проверяем, не вышел ли индекс за границы
            let new_tracks_len = tracks_len - 1;
            if self.current_index >= new_tracks_len && new_tracks_len > 0 {
                self.current_index = new_tracks_len - 1;
            } else if new_tracks_len == 0 {
                self.current_index = 0;
                self.stop();
                self.current_track = None;
            }
        }
        Ok(())
    }
    
    pub fn clear_current_playlist(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(current_playlist_name) = &self.current_playlist {
            if let Some(playlist) = self.playlists.get_mut(current_playlist_name) {
                playlist.clear_tracks()?;
                self.current_track = None;
                self.current_index = 0;
                self.stop();
            }
        }
        Ok(())
    }

    pub fn play(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.state {
            PlaybackState::Paused => {
                if let Some(ref sink) = self.sink {
                    sink.play();
                }
                self.state = PlaybackState::Playing;
                self.last_update = Instant::now();
            }
            PlaybackState::Stopped => {
                if let Some(track) = self.get_current_track() {
                    self.load_and_play_track(track)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            if let Some(ref sink) = self.sink {
                sink.pause();
            }
            self.state = PlaybackState::Paused;
        }
    }

    pub fn stop(&mut self) {
        if let Some(ref sink) = self.sink {
            sink.stop();
        }
        self.sink = None;
        self._stream = None;
        self.state = PlaybackState::Stopped;
        self.position = Duration::from_secs(0);
    }

    pub fn next(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(playlist_name) = &self.current_playlist {
            if let Some(playlist) = self.playlists.get(playlist_name) {
                if !playlist.tracks.is_empty() {
                    self.current_index = (self.current_index + 1) % playlist.tracks.len();
                    if let Some(track) = self.get_current_track() {
                        self.load_and_play_track(track)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn previous(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(playlist_name) = &self.current_playlist {
            if let Some(playlist) = self.playlists.get(playlist_name) {
                if !playlist.tracks.is_empty() {
                    self.current_index = if self.current_index == 0 {
                        playlist.tracks.len() - 1
                    } else {
                        self.current_index - 1
                    };
                    if let Some(track) = self.get_current_track() {
                        self.load_and_play_track(track)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if let Some(ref sink) = self.sink {
            sink.set_volume(self.volume);
        }
    }

    pub fn seek_to(&mut self, position: Duration) -> Result<(), Box<dyn std::error::Error>> {
        // Rodio не поддерживает прямую перемотку, поэтому перезагружаем трек
        if let Some(track) = self.current_track.clone() {
            self.load_and_play_track(track)?;
            self.position = position;
            // В реальном приложении здесь была бы более сложная логика перемотки
        }
        Ok(())
    }

    pub fn get_position(&self) -> Duration {
        self.position
    }

    fn get_current_track(&self) -> Option<Track> {
        if let Some(playlist_name) = &self.current_playlist {
            if let Some(playlist) = self.playlists.get(playlist_name) {
                return playlist.tracks.get(self.current_index).cloned();
            }
        }
        None
    }

    fn get_current_playlist(&self) -> Option<&Playlist> {
        if let Some(playlist_name) = &self.current_playlist {
            self.playlists.get(playlist_name)
        } else {
            None
        }
    }

    fn load_and_play_track(&mut self, track: Track) -> Result<(), Box<dyn std::error::Error>> {
        self.stop();
        
        let file = File::open(&track.path)?;
        let reader = BufReader::new(file);
        let source = Decoder::new(reader)?;
        
        // Создаем новый поток и sink для rodio 0.17
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        
        sink.append(source);
        sink.set_volume(self.volume);
        sink.play();
        
        self.sink = Some(sink);
        self._stream = Some(_stream);
        self.current_track = Some(track);
        self.state = PlaybackState::Playing;
        self.position = Duration::from_secs(0);
        self.last_update = Instant::now();
        
        Ok(())
    }

    pub fn update(&mut self) {
        if self.state == PlaybackState::Playing {
            let now = Instant::now();
            let elapsed = now.duration_since(self.last_update);
            self.position += elapsed;
            self.last_update = now;

            // Check if current track finished
            if let Some(ref sink) = self.sink {
                if sink.empty() {
                    if let Some(playlist) = self.get_current_playlist() {
                        if !playlist.tracks.is_empty() {
                            // Track finished, play next
                            let _ = self.next();
                        }
                    }
                }
            }
        }
    }
}

struct AudioPlayerApp {
    player: AudioPlayer,
}

impl Default for AudioPlayerApp {
    fn default() -> Self {
        let mut player = AudioPlayer::new().unwrap_or_else(|_| {
            panic!("Failed to initialize audio player");
        });
        
        // Загружаем плейлисты при запуске
        let _ = player.load_playlists();
        
        Self { player }
    }
}

impl eframe::App for AudioPlayerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.player.update();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎵 Audio Player");
            ui.separator();

            // Current track info
            if let Some(ref track) = self.player.current_track.clone() {
                ui.horizontal(|ui| {
                    ui.label("Playing:");
                    ui.label(RichText::new(&track.title).color(Color32::LIGHT_BLUE));
                });
                
                // Progress bar with time display and seeking
                let current_time = self.player.position;
                let track_duration = track.duration.unwrap_or(Duration::from_secs(180));
                
                ui.horizontal(|ui| {
                    let current_secs = current_time.as_secs();
                    
                    ui.label(format!("{:02}:{:02}", 
                        current_secs / 60,
                        current_secs % 60
                    ));
                    
                    // Используем реальную длительность трека из метаданных
                    let progress = current_time.as_secs_f32() / track_duration.as_secs_f32();
                    
                    let progress_bar = egui::ProgressBar::new(progress.clamp(0.0, 1.0))
                        .desired_width(200.0)
                        .animate(self.player.state == PlaybackState::Playing);
                    
                    let response = ui.add(progress_bar);
                    
                    // Добавляем возможность перемотки по клику
                    if response.clicked() {
                        if let Some(click_pos) = response.interact_pointer_pos() {
                            let rect = response.rect;
                            let relative_x = (click_pos.x - rect.left()) / rect.width();
                            let seek_time = Duration::from_secs_f32(
                                relative_x * track_duration.as_secs_f32()
                            );
                            let _ = self.player.seek_to(seek_time);
                        }
                    }
                    
                    let duration_secs = track_duration.as_secs();
                    ui.label(format!("{:02}:{:02}", 
                        duration_secs / 60,
                        duration_secs % 60
                    ));
                });
            } else {
                ui.label("No track selected");
            }

            ui.separator();

            // Control buttons
            ui.horizontal(|ui| {
                if ui.button("⏮").clicked() {
                    let _ = self.player.previous();
                }
                
                // Перемотка назад на 10 секунд
                if ui.button("⏪").clicked() {
                    let current_pos = self.player.get_position();
                    let new_pos = if current_pos >= Duration::from_secs(10) {
                        current_pos - Duration::from_secs(10)
                    } else {
                        Duration::from_secs(0)
                    };
                    let _ = self.player.seek_to(new_pos);
                }
                
                match self.player.state {
                    PlaybackState::Playing => {
                        if ui.button("⏸").clicked() {
                            self.player.pause();
                        }
                    }
                    _ => {
                        if ui.button("▶").clicked() {
                            let _ = self.player.play();
                        }
                    }
                }
                
                // Перемотка вперед на 10 секунд
                if ui.button("⏩").clicked() {
                    let current_pos = self.player.get_position();
                    let new_pos = current_pos + Duration::from_secs(10);
                    let _ = self.player.seek_to(new_pos);
                }
                
                if ui.button("⏹").clicked() {
                    self.player.stop();
                }
                
                if ui.button("⏭").clicked() {
                    let _ = self.player.next();
                }
            });

            ui.separator();

            // Volume control
            ui.horizontal(|ui| {
                ui.label("Volume:");
                ui.add(egui::Slider::new(&mut self.player.volume, 0.0..=1.0).step_by(0.01));
                self.player.set_volume(self.player.volume);
            });

            ui.separator();

            // Playlist management
            ui.horizontal(|ui| {
                if ui.button("Reload Playlists").clicked() {
                    let _ = self.player.load_playlists();
                }
                
                if ui.button("Create Playlist").clicked() {
                    self.player.show_playlist_dialog = true;
                }
                
                if ui.button("Add Track").clicked() {
                    self.player.show_add_track_dialog = true;
                }
                
                if ui.button("Music Folder Info").clicked() {
                    self.player.show_file_dialog = true;
                }
            });

            // Current playlist selector
            ui.horizontal(|ui| {
                ui.label("Current Playlist:");
                let current_playlist = self.player.current_playlist.clone();
                if let Some(current_playlist_name) = current_playlist {
                    let playlist_names: Vec<String> = self.player.playlists.keys().cloned().collect();
                    egui::ComboBox::from_label("")
                        .selected_text(&current_playlist_name)
                        .show_ui(ui, |ui| {
                            for playlist_name in playlist_names {
                                if ui.selectable_value(&mut self.player.current_playlist, 
                                    Some(playlist_name.clone()), &playlist_name).clicked() {
                                    self.player.current_index = 0;
                                    self.player.stop();
                                }
                            }
                        });
                    
                    if ui.button("Clear Playlist").clicked() {
                        let _ = self.player.clear_current_playlist();
                    }
                } else {
                    ui.label("No playlist selected");
                }
            });

            // Music folder info dialog
            if self.player.show_file_dialog {
                egui::Window::new("Music Folder Info")
                    .collapsible(false)
                    .resizable(true)
                    .show(ctx, |ui| {
                        ui.label("Music folder location:");
                        ui.label(format!("{}", self.player.music_folder.display()));
                        ui.separator();
                        
                        ui.label("Instructions:");
                        ui.label("1. Create playlists to organize your music");
                        ui.label("2. Add tracks to playlists using 'Add Track' button");
                        ui.label("3. Tracks are automatically copied to playlist folders");
                        ui.label("4. Supported formats: MP3, WAV, FLAC, OGG");
                        
                        if ui.button("Refresh Playlists").clicked() {
                            let _ = self.player.load_playlists();
                        }
                        
                        if ui.button("Close").clicked() {
                            self.player.show_file_dialog = false;
                        }
                    });
            }

            // Create playlist dialog
            if self.player.show_playlist_dialog {
                egui::Window::new("Create Playlist")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label("Playlist name:");
                        ui.text_edit_singleline(&mut self.player.new_playlist_name);
                        
                        ui.horizontal(|ui| {
                            if ui.button("Create").clicked() {
                                if !self.player.new_playlist_name.trim().is_empty() {
                                    let _ = self.player.create_playlist(self.player.new_playlist_name.trim().to_string());
                                    self.player.new_playlist_name.clear();
                                    self.player.show_playlist_dialog = false;
                                }
                            }
                            
                            if ui.button("Cancel").clicked() {
                                self.player.new_playlist_name.clear();
                                self.player.show_playlist_dialog = false;
                            }
                        });
                    });
            }

            // Add track dialog
            if self.player.show_add_track_dialog {
                egui::Window::new("Add Track")
                    .collapsible(false)
                    .resizable(true)
                    .show(ctx, |ui| {
                        ui.label("Add tracks to current playlist:");
                        if let Some(current_playlist) = &self.player.current_playlist {
                            ui.label(format!("Current playlist: {}", current_playlist));
                            if let Some(playlist) = self.player.playlists.get(current_playlist) {
                                ui.label(format!("Folder: {}", playlist.folder_path.display()));
                            }
                        }
                        ui.separator();
                        
                        ui.horizontal(|ui| {
                            if ui.button("Select Files").clicked() {
                                let _ = self.player.select_and_add_tracks();
                                self.player.show_add_track_dialog = false;
                            }
                            
                            if ui.button("Cancel").clicked() {
                                self.player.show_add_track_dialog = false;
                            }
                        });
                        
                        ui.separator();
                        ui.label("Supported formats: MP3, WAV, FLAC, OGG");
                        ui.label("Files will be copied to the playlist folder.");
                    });
            }

            ui.separator();

            // Playlist
            let current_playlist_name = self.player.current_playlist.clone();
            if let Some(current_playlist_name) = current_playlist_name {
                ui.heading(format!("Playlist: {}", current_playlist_name));
                
                let tracks: Vec<Track> = if let Some(playlist) = self.player.playlists.get(&current_playlist_name) {
                    playlist.tracks.iter().cloned().collect()
                } else {
                    Vec::new()
                };
                
                if tracks.is_empty() {
                    ui.label("No tracks in playlist. Add tracks using 'Add Track' button.");
                } else {
                    let current_index = self.player.current_index;
                    let current_state = self.player.state;
                    
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut clicked_index = None;
                        let mut remove_index = None;
                        
                        for (index, track) in tracks.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let is_current = index == current_index;
                                let text_color = if is_current { 
                                    Color32::LIGHT_BLUE 
                                } else { 
                                    Color32::WHITE 
                                };
                                
                                if ui.button("▶").clicked() {
                                    clicked_index = Some(index);
                                }
                                
                                ui.label(RichText::new(&track.title).color(text_color));
                                
                                if is_current && current_state == PlaybackState::Playing {
                                    ui.label("🔊");
                                }
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("🗑").on_hover_text("Delete track").clicked() {
                                        remove_index = Some(index);
                                    }
                                });
                            });
                        }
                        
                        if let Some(index) = clicked_index {
                            self.player.current_index = index;
                            let _ = self.player.play();
                        }
                        
                        if let Some(index) = remove_index {
                            let _ = self.player.remove_track_from_current_playlist(index);
                        }
                    });
                }
            } else {
                ui.heading("No Playlist Selected");
            }
        });

        // Request continuous updates when playing
        if self.player.state == PlaybackState::Playing {
            ctx.request_repaint();
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Audio Player",
        options,
        Box::new(|_cc| Ok(Box::new(AudioPlayerApp::default()))),
    )
}