use rodio::{Decoder, MixerDeviceSink, Player};
use std::{io::Cursor, time::Duration};
use winit::keyboard::KeyCode;

use crate::core;

static AUDIO: &[u8] = include_bytes!("../../../../assets/engine.wav");

pub struct AudioSystem {
    sink: Player,
    _stream_handle: MixerDeviceSink,
}

impl AudioSystem {
    pub fn new() -> Self {
        let mut stream_handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        stream_handle.log_on_drop(false);
        let sink = rodio::Player::connect_new(stream_handle.mixer());
        sink.pause();
        if crate::core::MUTE {
            sink.set_volume(0.0);
        } else {
            sink.set_volume(0.2);
        }
        sink.append(
            // TODO: Currently hardcoded to example audio.
            Decoder::new(Cursor::new(AUDIO)).unwrap(),
        );

        Self {
            sink,
            _stream_handle: stream_handle,
        }
    }
}

impl core::System for AudioSystem {
    fn before_tick(&mut self, args: &mut core::BeforeTickArgs) {
        if *args.input.is_pressed(&KeyCode::KeyW)
            | *args.input.is_pressed(&KeyCode::KeyA)
            | *args.input.is_pressed(&KeyCode::KeyS)
            | *args.input.is_pressed(&KeyCode::KeyD)
        {
            if *args.input.is_pressed(&KeyCode::ControlLeft) {
                self.sink.set_speed(2.0);
            } else {
                self.sink.set_speed(1.0);
            }
            self.sink.play();
            if self.sink.get_pos() > Duration::new(5, 0) {
                // TODO: Fix
                // self.sink.try_seek(Duration::ZERO).unwrap();
            }
        } else {
            self.sink.set_speed(1.0);
            self.sink.pause();
        }
    }
}
