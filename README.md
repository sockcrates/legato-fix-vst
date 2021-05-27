Legato Fix VST
===

This is a VST to fix legato issues in a MIDI to CV converter. Mac OS requires special steps to compile. This
plugin will hold any new notes triggered before the original note's "note off" event.

Converts MIDI notes of the form:
~~~
[___]
   [________]
        [_______]
~~~

To the form:
~~~
[_______________]
   [____________]
        [_______]
~~~

Where "```[```" is a note on event and "```]```" is a note off event.

### Usage
Use the compiled VST with your DAW and have it output MIDI to your MIDI/CV converter.

### Packaging on OS X
Refer to the instructions in the vst-rs [GitHub docs](https://github.com/RustAudio/vst-rs#packaging-on-os-x).
