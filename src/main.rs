use std::{
    fs,
    time::{Duration, Instant},
};

use symphonia::core::{
    codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL},
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};
use symphonia::core::{errors::Error, formats::FormatReader};

enum AudioState {
    Ready,
    Playing,
}
struct PlayerData {
    audio_stream_reader: Box<dyn FormatReader>,
    track_id: u32,
    decoder: Box<dyn Decoder>,
}
struct AppState {
    audio_state: AudioState,
    audio_path: String,
    fps_as_ms: u64,
    player_data: Option<PlayerData>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            audio_state: AudioState::Ready,
            player_data: None,
            audio_path: "./sample.mp3".to_string(),
            fps_as_ms: 41, // 24 FPS -> https://fpstoms.com/
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.audio_state {
                AudioState::Ready => {
                    let src =
                        fs::File::open(self.audio_path.clone()).expect("failed to open media");
                    let mss = MediaSourceStream::new(Box::new(src), Default::default());
                    let mut hint = Hint::new();
                    hint.with_extension("mp3");
                    let dec_opts: DecoderOptions = Default::default();
                    let meta_opts: MetadataOptions = Default::default();
                    let fmt_opts: FormatOptions = Default::default();

                    // Probe the media source.
                    let probed = symphonia::default::get_probe()
                        .format(&hint, mss, &fmt_opts, &meta_opts)
                        .expect("unsupported format");

                    // Get the instantiated format reader.
                    let audio_stream_reader = probed.format;

                    // Find the first audio track with a known (decodeable) codec.
                    let track = audio_stream_reader
                        .tracks()
                        .iter()
                        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
                        .expect("no supported audio tracks");
                    let decoder = symphonia::default::get_codecs()
                        .make(&track.codec_params, &dec_opts)
                        .expect("unsupported codec");

                    let track_id = track.id;
                    self.audio_state = AudioState::Playing;
                    self.player_data = Some(PlayerData {
                        audio_stream_reader,
                        track_id,
                        decoder,
                    });
                }
                AudioState::Playing => {
                    let player_data = self.player_data.as_mut().unwrap();
                    let packet = match player_data.audio_stream_reader.next_packet() {
                        Ok(p) => p,
                        Err(Error::IoError(err)) => {
                            panic!("{}", err);
                        }
                        Err(Error::DecodeError(err)) => {
                            panic!("{}", err);
                        }
                        Err(err) => {
                            panic!("{}", err);
                        }
                    };

                    // Consume any new metadata that has been read since the last packet.
                    while !player_data.audio_stream_reader.metadata().is_latest() {
                        // Pop the old head of the metadata queue.
                        player_data.audio_stream_reader.metadata().pop();
                        if let Some(rev) = player_data.audio_stream_reader.metadata().current() {
                            // Consume the new metadata at the head of the metadata queue.
                            println!("{:?}", rev);
                        }
                    }

                    // If the packet does not belong to the selected track, skip over it.
                    if packet.track_id() != player_data.track_id {
                        panic!(
                            "wrong track_id current: {}; in state: {}",
                            packet.track_id(),
                            player_data.track_id
                        );
                    }
                    // Decode the packet into audio samples.
                    match player_data.decoder.as_mut().decode(&packet) {
                        Ok(decoded) => {
                            // Consume the decoded audio samples (see below).
                            // println!("{:?}", decoded.frames());
                        }
                        Err(err) => {
                            // An unrecoverable error occurred, halt decoding.
                            panic!("{}", err);
                        }
                    }
                }
            }
            ui.heading("audio_visualizer");
            ui.label("Drag-and-drop files onto the window!");
            ui.label(format!("Playing '{}'", self.audio_path));
            ui.label(format!("Time: {:?}", Instant::now()));
            ctx.request_repaint_after(Duration::from_millis(self.fps_as_ms));
        });
    }
}

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 640.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "audio_visualizer",
        native_options,
        Box::new(|_cc| Ok(Box::<AppState>::default())),
    );
}
