use crate::car::CarComponent;
use glui::mecs::*;
use glui::tools::{Camera, Vec3};
use rodio::{Sample, Sink, Source, SpatialSink};
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

#[derive(Clone, Debug)]
pub enum SoundMsg {
    MusicSpeed(f32),
    Engine(Vec<(f32, f32, Vec3, Vec3, Vec3)>),
    PlayMusic(bool),
    Stop,
}

pub struct Sounds {
    thread: Option<JoinHandle<()>>,
    sender: Sender<SoundMsg>,
    cars: Vec<Entity>,
    camera: Entity,
    music: bool,
}

impl System for Sounds {
    fn update(&mut self, _delta_time: Duration, world: &mut StaticWorld) {
        let car = world.component::<CarComponent>(self.cars[0]).unwrap();
        let s = car.speed();
        self.sender
            .send(SoundMsg::MusicSpeed((s * 0.01) * (s * 0.01) * 0.05 + 1.0))
            .unwrap_or_default();

        let cam = world
            .component::<DataComponent<Camera>>(self.camera)
            .unwrap()
            .data
            .params
            .spatial;

        let mut specs = vec![];

        for car in self.cars.iter() {
            let car = world.component::<CarComponent>(*car).unwrap();
            let throttle = car.throttle;
            let speed = throttle * 1.5 + 0.5;
            let volume = throttle * 2.0 + 4.0;

            specs.push((
                speed,
                volume,
                car.pos3(),
                cam.pos - cam.r() * 2.0,
                cam.pos + cam.r() * 2.0,
            ));
        }
        self.sender
            .send(SoundMsg::Engine(specs))
            .unwrap_or_default();
    }

    fn window_event(&mut self, event: &GlutinWindowEvent, _world: &mut StaticWorld) -> bool {
        if let GlutinWindowEvent::KeyboardInput { input, .. } = event {
            let press = input.state == GlutinElementState::Pressed;
            if let Some(key) = input.virtual_keycode {
                if key == GlutinKey::M && !press {
                    self.music = !self.music;
                    self.sender
                        .send(SoundMsg::PlayMusic(self.music))
                        .unwrap_or_default();
                }
            }
        }

        false
    }
}

impl Sounds {
    pub fn new(cars: Vec<Entity>, camera: Entity, muted: bool) -> Sounds {
        let (tx, rx) = mpsc::channel();
        let n = cars.len();

        let t = thread::spawn(move || {
            let device = rodio::default_output_device().unwrap();

            let file = File::open("sounds/music.mp3").unwrap();
            let source = rodio::Decoder::new(BufReader::new(file))
                .unwrap()
                .buffered()
                .repeat_infinite();
            let source = AdjustableSpeed {
                input: source,
                factor: Arc::new(AtomicI32::new(10000)),
            };
            let music_speed = source.factor.clone();
            let music_sink = Sink::new(&device);
            music_sink.append(source);
            music_sink.set_volume(0.25);

            if muted {
                music_sink.pause();
            }

            let mut engines = vec![];

            for _ in 0..n {
                let file = File::open("sounds/engine.mp3").unwrap();
                let source = rodio::Decoder::new(BufReader::new(file))
                    .unwrap()
                    .buffered()
                    .repeat_infinite();
                let source = AdjustableSpeed {
                    input: source,
                    factor: Arc::new(AtomicI32::new(10000)),
                };
                let engine_speed = source.factor.clone();
                let engine_sink = SpatialSink::new(&device, [0.0; 3], [0.0; 3], [0.0; 3]);
                engine_sink.append(source);
                engine_sink.set_volume(0.1);

                engines.push((engine_speed, engine_sink));
            }

            let file = File::open("sounds/wind.mp3").unwrap();
            let source = rodio::Decoder::new(BufReader::new(file))
                .unwrap()
                .buffered()
                .repeat_infinite();
            let wind_sink = Sink::new(&device);
            wind_sink.append(source);
            wind_sink.set_volume(0.1);

            let mut running = true;
            while running {
                match rx.recv() {
                    Ok(msg) => match msg {
                        SoundMsg::Stop => {
                            running = false;
                        }
                        SoundMsg::MusicSpeed(f) => {
                            music_speed.store((f * 10000.0) as i32, Ordering::SeqCst);
                        }
                        SoundMsg::Engine(specs) => {
                            let mut i = 0;
                            for (speed, volume, emitter, left, right) in specs {
                                let (engine_speed, engine_sink) = &mut engines[i];
                                engine_speed.store((speed * 10000.0) as i32, Ordering::SeqCst);
                                engine_sink.set_volume(volume);
                                engine_sink.set_emitter_position(emitter.as_array());
                                engine_sink.set_left_ear_position(left.as_array());
                                engine_sink.set_right_ear_position(right.as_array());
                                i += 1;
                            }
                        }
                        SoundMsg::PlayMusic(play) => {
                            if play {
                                music_sink.play();
                            } else {
                                music_sink.pause();
                            }
                        }
                    },
                    Err(e) => {
                        println!("Sound died: {:?}", e);
                        running = false;
                    }
                }
            }
        });

        Sounds {
            thread: Some(t),
            sender: tx,
            cars,
            camera,
            music: !muted,
        }
    }
}

impl Drop for Sounds {
    fn drop(&mut self) {
        self.sender.send(SoundMsg::Stop).unwrap_or_default();
        self.thread.take().map(JoinHandle::join);
    }
}

#[derive(Clone, Debug)]
pub struct AdjustableSpeed<I> {
    input: I,
    factor: Arc<AtomicI32>,
}

impl<I> Iterator for AdjustableSpeed<I>
where
    I: Source,
    I::Item: Sample,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        self.input.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.input.size_hint()
    }
}

impl<I> ExactSizeIterator for AdjustableSpeed<I>
where
    I: Source + ExactSizeIterator,
    I::Item: Sample,
{
}

impl<I> Source for AdjustableSpeed<I>
where
    I: Source,
    I::Item: Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.input.channels()
    }

    fn sample_rate(&self) -> u32 {
        let f = self.factor.load(Ordering::SeqCst) as f32 / 10000.0;
        (self.input.sample_rate() as f32 * f) as u32
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        if let Some(duration) = self.input.total_duration() {
            let f = self.factor.load(Ordering::SeqCst) as f32 / 10000.0;

            let as_ns = duration.as_secs() * 1000000000 + duration.subsec_nanos() as u64;
            let new_val = (as_ns as f32 / f) as u64;
            Some(Duration::new(
                new_val / 1000000000,
                (new_val % 1000000000) as u32,
            ))
        } else {
            None
        }
    }
}
