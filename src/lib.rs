#[macro_use]
extern crate vst;

use vst::plugin::{HostCallback, Info, Plugin};

#[derive(Default)]
struct PM01Plugin;

impl Plugin for PM01Plugin {
    fn new(_host: HostCallback) -> Self {
        PM01Plugin
    }

    fn get_info(&self) -> Info {
        Info {
            inputs: 0,
            outputs: 0,
            name: "PM01 Legato Fix".to_string(),
            unique_id: 52468,
            ..Default::default()
        }
    }
}

plugin_main!(PM01Plugin);
