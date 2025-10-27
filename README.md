# Camera Controls APP

A generic capture controller for USB and/or Bluetooth webcam devices.

Leverages the following backends:

Backend      |	Platform
-------------|-----------
v4l2	       |	Linux
MSMF         |	Windows
AVFoundation |	Darwin (MacOS)

The following actions are supported:
 - pan
 - tilt
 - zoom

**NOTE**: more to add in the future

## Usage

The application comes with a simple UI, which makes it possible to adjust each action with a slider
It is also possible to adjust it via shortcuts, which are enabled by default.

Webcam Action  |	Shortcut
---------------|-------------
Move left      | `Numpad 4`
Move right     | `Numpad 6`
Move Up        | `Numpad 8`
Move Down      | `Numpad 2`
Zoom In        | `Numpad +`
Zoom Out       | `Numpad -`


The application can also save settings by profile up to 5.

Profile Action |	Shortcut
---------------|-------------
Profile 1      | `Ctrl + 1`
...     2      | `Ctrl + 2`
...     3      | `Ctrl + 3`
...     4      | `Ctrl + 4`
...     5      | `Ctrl + 5`
