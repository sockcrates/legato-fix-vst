#[macro_use]
extern crate vst;

use std::cell::Cell;
use vst::api::Events;
use vst::buffer::{AudioBuffer, SendEventBuffer};
use vst::event::Event;
use vst::plugin::{CanDo, Category, HostCallback, Info, Plugin};

#[derive(Default)]
struct LegatoFixPlugin {
    host: HostCallback,
    notes: Cell<u8>,
    recv_buffer: SendEventBuffer,
    send_buffer: SendEventBuffer,
}

impl LegatoFixPlugin {
    fn handle_events(&mut self, events: &Events) {
        let mut notes = self.notes.get();
        let fixed_events = events.events().filter(|event| {
            match event {
                Event::Midi(ev) => match ev.data[0] {
                    // Note on
                    144 => {
                        notes += 1u8;
                        true
                    }
                    // Note off
                    128 => {
                        notes -= 1u8;

                        if notes > 0u8 {
                            return false;
                        }

                        true
                    }
                    _ => true,
                },
                _ => true,
            }
        });

        self.recv_buffer.store_events(fixed_events);
        self.notes.set(notes);
    }

    fn send_midi(&mut self) {
        self.send_buffer
            .send_events(self.recv_buffer.events().events(), &mut self.host);
        self.recv_buffer.clear();
    }
}

impl Plugin for LegatoFixPlugin {
    fn new(host: HostCallback) -> Self {
        LegatoFixPlugin {
            host,
            notes: Cell::new(0),
            ..Default::default()
        }
    }

    fn get_info(&self) -> Info {
        Info {
            category: Category::Synth,
            midi_inputs: 1,
            midi_outputs: 1,
            name: "Legato Fix".to_string(),
            unique_id: 25624,
            ..Default::default()
        }
    }

    fn process_events(&mut self, events: &Events) {
        self.handle_events(events);
    }

    fn process(&mut self, _buffer: &mut AudioBuffer<f32>) {
        self.send_midi();
    }

    fn can_do(&self, can_do: CanDo) -> vst::api::Supported {
        use vst::api::Supported::*;
        use vst::plugin::CanDo::*;

        match can_do {
            SendEvents | SendMidiEvent | ReceiveEvents | ReceiveMidiEvent => Yes,
            _ => No,
        }
    }
}

plugin_main!(LegatoFixPlugin);
