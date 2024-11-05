use std::{
    fs,
    time::{Duration, Instant},
};

use egui::{epaint::CircleShape, Color32, Mesh, Pos2, Rect, Shape};
use symphonia::core::{
    audio::SampleBuffer,
    codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL},
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};
use symphonia::core::{errors::Error, formats::FormatReader};

mod num_to_circle;
mod num_to_color;

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
    sample_count: usize,
    sample_buf: Option<SampleBuffer<f32>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            audio_state: AudioState::Ready,
            player_data: None,
            audio_path: "./sample.mp3".to_string(),
            fps_as_ms: 41, // 24 FPS -> https://fpstoms.com/
            sample_count: 0,
            sample_buf: None,
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
                            println!("rev {:?}", rev);
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
                            if self.sample_buf.is_none() {
                                // Get the audio buffer specification.
                                let spec = *decoded.spec();

                                // Get the capacity of the decoded buffer. Note: This is capacity, not length!
                                let duration = decoded.capacity() as u64;

                                // Create the f32 sample buffer.
                                self.sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                            }

                            // Copy the decoded audio buffer into the sample buffer in an interleaved format.
                            if let Some(buf) = &mut self.sample_buf {
                                buf.copy_interleaved_ref(decoded);

                                // The samples may now be access via the `samples()` function.
                                self.sample_count += buf.samples().len();
                                print!("\rDecoded {} samples", self.sample_count);
                            }
                        }
                        Err(err) => {
                            // An unrecoverable error occurred, halt decoding.
                            panic!("{}", err);
                        }
                    }
                }
            }
            let is_empty = self.sample_buf.as_mut().is_none();

            if !is_empty {
                let samples = self.sample_buf.as_mut().unwrap().samples();
                let color = num_to_color::number_to_color(samples.last());
                let sample_len = samples.len();
                let label = format!("Sample len: {}", samples.len());
                ui.label(egui::RichText::new(label).heading().color(color));
                let mut rect = Rect::from_pos(egui::Pos2 { x: 0.0, y: 0.0 });
                rect.set_height(640.0);
                rect.set_width(640.0);
                let mut mesh = Mesh::default();
                mesh.add_colored_rect(rect, color);
                ui.painter().add(Shape::mesh(mesh));

                let (r1, color_1) = num_to_circle::number_to_circle(samples.first());
                let c1 = CircleShape {
                    center: Pos2 { x: 160.0, y: 160.0 },
                    radius: r1,
                    fill: color_1,
                    stroke: egui::Stroke {
                        width: 1.0,
                        color: Color32::RED,
                    },
                };
                ui.painter().add(c1);

                let (r2, color_2) = num_to_circle::number_to_circle(samples.get(sample_len / 4));
                let c2 = CircleShape {
                    center: Pos2 { x: 480.0, y: 160.0 },
                    radius: r2,
                    fill: color_2,
                    stroke: egui::Stroke {
                        width: 1.0,
                        color: Color32::RED,
                    },
                };
                ui.painter().add(c2);

                let (r3, color_3) = num_to_circle::number_to_circle(samples.get(sample_len / 2));
                let c3 = CircleShape {
                    center: Pos2 { x: 160.0, y: 480.0 },
                    radius: r3,
                    fill: color_3,
                    stroke: egui::Stroke {
                        width: 1.0,
                        color: Color32::RED,
                    },
                };
                ui.painter().add(c3);

                let (r4, color_4) =
                    num_to_circle::number_to_circle(samples.get(sample_len / 2 + sample_len / 4));
                let c4 = CircleShape {
                    center: Pos2 { x: 480.0, y: 480.0 },
                    radius: r4,
                    fill: color_4,
                    stroke: egui::Stroke {
                        width: 1.0,
                        color: Color32::RED,
                    },
                };
                ui.painter().add(c4);
            }

            ui.heading("audio_visualizer");
            ui.label("Drag-and-drop files onto the window!");
            ui.label(format!("Playing '{}'", self.audio_path));
            ui.label(format!("Time: {:?}", Instant::now()));
            ui.label(format!("sample_count: {:?}", self.sample_count));

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
