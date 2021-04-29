#[macro_use]
extern crate vst;

use vst::api::Events;
use vst::event::Event;
use vst::plugin::{Category, HostCallback, Info, Plugin};

#[derive(Default)]
struct PM01Plugin {
    notes: u8,
}

impl Plugin for PM01Plugin {
    fn new(_host: HostCallback) -> Self {
        PM01Plugin { notes: 0 }
    }

    fn get_info(&self) -> Info {
        Info {
            category: Category::Effect,
            midi_inputs: 0,
            midi_outputs: 0,
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
                    144 => self.notes += 1u8,
                    // Note off
                    128 => self.notes -= 1u8,
                    _ => (),
                },
                _ => (),
            }
        }
    }
}

plugin_main!(PM01Plugin);
