#[macro_use]
extern crate vst;

use vst::api::Events;
use vst::buffer::{AudioBuffer, SendEventBuffer};
use vst::event::{Event, MidiEvent};
use vst::plugin::{CanDo, Category, HostCallback, Info, Plugin};

#[derive(Default)]
struct LegatoFixPlugin {
    host: HostCallback,
    notes: i8,
    note_off_data: Vec<MidiEvent>,
    recv_buffer: SendEventBuffer,
    send_buffer: SendEventBuffer,
}

plugin_main!(LegatoFixPlugin);

impl LegatoFixPlugin {
    fn handle_events(&mut self, events: &Events) {
        let mut fixed_events = vec![];

        for event in events.events() {
            match event {
                Event::Midi(ev) => match ev.data[0] {
                    // Note on
                    144 => {
                        self.notes += 1i8;
                        fixed_events.push(event);
                    }
                    // Note off
                    128 => {
                        self.note_off_data.push(ev);
                        self.notes -= 1i8;
                    }
                    _ => fixed_events.push(event),
                },
                _ => fixed_events.push(event),
            }
        }

        self.recv_buffer.store_events(fixed_events);
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

        if self.notes <= 0 {
            self.recv_buffer.store_events(self.note_off_data.clone());
            self.note_off_data.clear();
            self.notes = 0;
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
