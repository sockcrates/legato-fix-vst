#[macro_use]
extern crate vst;

use vst::api::Events;
use vst::buffer::{AudioBuffer, SendEventBuffer};
use vst::event::{Event, MidiEvent};
use vst::plugin::{CanDo, Category, HostCallback, Info, Plugin};

#[derive(Default)]
struct LegatoFixPlugin {
    host: HostCallback,
    notes: usize,
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
                Event::Midi(ev) => match ev.data[0] & 0xf0 {
                    // Note on with a non-zero velocity.
                    0x90 if ev.data[2] != 0 => {
                        self.notes = self.notes.saturating_add(1);
                        fixed_events.push(event);
                    }
                    // Note off, including a note-on with zero velocity.
                    0x80 | 0x90 => {
                        self.note_off_data.push(ev);
                        self.notes = self.notes.saturating_sub(1);

                        if self.notes == 0 {
                            fixed_events.extend(self.note_off_data.drain(..).map(Event::Midi));
                        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn midi(status: u8, note: u8, velocity: u8) -> MidiEvent {
        MidiEvent {
            data: [status, note, velocity],
            delta_frames: 0,
            live: false,
            note_length: None,
            note_offset: None,
            detune: 0,
            note_off_velocity: 0,
        }
    }

    fn process(plugin: &mut LegatoFixPlugin, events: Vec<MidiEvent>) {
        let mut input = SendEventBuffer::new(events.len());
        input.store_events(events);
        plugin.process_events(input.events());
    }

    fn queued_events(plugin: &LegatoFixPlugin) -> Vec<[u8; 3]> {
        plugin
            .recv_buffer
            .events()
            .events()
            .map(|event| match event {
                Event::Midi(event) => event.data,
                _ => panic!("test input only contains MIDI events"),
            })
            .collect()
    }

    #[test]
    fn holds_note_offs_until_the_final_active_note_is_released() {
        let mut plugin = LegatoFixPlugin::default();
        let on_e = midi(0x90, 64, 100);
        let on_d_sharp = midi(0x90, 63, 100);
        let off_e = midi(0x80, 64, 0);
        let off_d_sharp = midi(0x80, 63, 0);

        process(&mut plugin, vec![on_e]);
        assert_eq!(queued_events(&plugin), vec![on_e.data]);
        assert_eq!(plugin.notes, 1);

        process(&mut plugin, vec![on_d_sharp, off_e]);
        assert_eq!(queued_events(&plugin), vec![on_d_sharp.data]);
        assert_eq!(plugin.notes, 1);
        assert_eq!(plugin.note_off_data.len(), 1);

        process(&mut plugin, vec![off_d_sharp]);
        assert_eq!(queued_events(&plugin), vec![off_e.data, off_d_sharp.data]);
        assert_eq!(plugin.notes, 0);
        assert!(plugin.note_off_data.is_empty());
    }

    #[test]
    fn forwards_non_note_midi_events_without_changing_note_state() {
        let mut plugin = LegatoFixPlugin::default();
        let control_change = midi(0xB0, 1, 64);

        process(&mut plugin, vec![control_change]);

        assert_eq!(queued_events(&plugin), vec![control_change.data]);
        assert_eq!(plugin.notes, 0);
        assert!(plugin.note_off_data.is_empty());
    }

    #[test]
    fn orphan_note_off_is_forwarded_and_note_count_is_clamped() {
        let mut plugin = LegatoFixPlugin::default();
        let note_off = midi(0x80, 64, 0);

        process(&mut plugin, vec![note_off]);

        assert_eq!(queued_events(&plugin), vec![note_off.data]);
        assert_eq!(plugin.notes, 0);
        assert!(plugin.note_off_data.is_empty());
    }

    #[test]
    fn recognizes_note_messages_on_all_midi_channels() {
        let mut plugin = LegatoFixPlugin::default();
        let note_on = midi(0x91, 64, 100);
        let note_off = midi(0x81, 64, 0);

        process(&mut plugin, vec![note_on]);
        assert_eq!(plugin.notes, 1);

        process(&mut plugin, vec![note_off]);
        assert_eq!(queued_events(&plugin), vec![note_off.data]);
        assert_eq!(plugin.notes, 0);
    }

    #[test]
    fn treats_zero_velocity_note_on_as_a_note_off() {
        let mut plugin = LegatoFixPlugin::default();
        let note_on = midi(0x90, 64, 100);
        let note_off = midi(0x90, 64, 0);

        process(&mut plugin, vec![note_on]);
        process(&mut plugin, vec![note_off]);

        assert_eq!(queued_events(&plugin), vec![note_off.data]);
        assert_eq!(plugin.notes, 0);
    }

    #[test]
    fn flushes_deferred_note_offs_without_reordering_later_events() {
        let mut plugin = LegatoFixPlugin::default();
        let note_on = midi(0x90, 64, 100);
        let note_off = midi(0x80, 64, 0);
        let control_change = midi(0xB0, 1, 64);

        process(&mut plugin, vec![note_on]);
        process(&mut plugin, vec![note_off, control_change]);

        assert_eq!(
            queued_events(&plugin),
            vec![note_off.data, control_change.data]
        );
    }

    #[test]
    fn tracks_more_than_a_signed_byte_of_active_notes() {
        let mut plugin = LegatoFixPlugin::default();

        for note in 0..129 {
            process(&mut plugin, vec![midi(0x90, note, 100)]);
        }

        assert_eq!(plugin.notes, 129);
    }

    #[test]
    fn empty_event_block_keeps_held_note_state_and_has_no_output() {
        let mut plugin = LegatoFixPlugin::default();
        process(&mut plugin, vec![midi(0x90, 64, 100)]);

        process(&mut plugin, vec![]);

        assert!(queued_events(&plugin).is_empty());
        assert_eq!(plugin.notes, 1);
        assert!(plugin.note_off_data.is_empty());
    }
}
