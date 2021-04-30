#[macro_use]
extern crate vst;

use vst::api::Events;
use vst::buffer::{AudioBuffer, SendEventBuffer};
use vst::event::Event;
use vst::plugin::{CanDo, Category, HostCallback, Info, Plugin};

#[derive(Default)]
struct PM01Plugin {
    host: HostCallback,
    notes: u8,
    recv_buffer: SendEventBuffer,
    send_buffer: SendEventBuffer,
}

impl PM01Plugin {
    fn send_midi(&mut self) {
        self.send_buffer
            .send_events(self.recv_buffer.events().events(), &mut self.host);
        self.recv_buffer.clear();
    }
}

impl Plugin for PM01Plugin {
    fn new(host: HostCallback) -> Self {
        PM01Plugin {
            host,
            notes: 0,
            ..Default::default()
        }
    }

    fn get_info(&self) -> Info {
        Info {
            category: Category::Synth,
            midi_inputs: 1,
            midi_outputs: 1,
            name: "PM01 Legato Fix".to_string(),
            unique_id: 25624,
            ..Default::default()
        }
    }

    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => match ev.data[0] {
                    // Note on
                    144 => {
                        self.notes += 1u8;
                        // Better way to store this event?
                        self.recv_buffer.store_events(vec![ev]);
                    }
                    // Note off
                    128 => {
                        self.notes -= 1u8;

                        if self.notes == 0u8 {
                            // Better way to store this event?
                            self.recv_buffer.store_events(vec![ev]);
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }
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

plugin_main!(PM01Plugin);
