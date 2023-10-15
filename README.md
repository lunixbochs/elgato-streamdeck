# elgato-streamdeck
Library for interacting with Elgato Stream Decks through [hidapi](https://crates.io/crates/hidapi). 

## udev rules for Linux
On Linux, you may need to install udev rules and make sure you're in the `plugdev` group:

```shell
cp 40-streamdeck.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

## Example
```rust
// Create instance of HidApi
let hid = new_hidapi();

// List devices and unsafely take first one
let (kind, serial) = StreamDeck::list_devices(&hid).remove(0);

// Connect to the device
let mut device = StreamDeck::connect(&hid, kind, &serial)
    .expect("Failed to connect");

// Print out some info from the device
println!(
    "Connected to '{}' with version '{}'",
    device.serial_number().unwrap(),
    device.firmware_version().unwrap()
);

// Set device brightness
device.set_brightness(35).unwrap();

// Use image-rs to load an image
let image = open("image.jpg").unwrap();

// Write it to the device
device.set_button_image(7, image).unwrap();
```

## Supported Devices
- Stream Deck Original
- Stream Deck Original V2
- Stream Deck XL
- Stream Deck XL V2
- Stream Deck Mini
- Stream Deck Mini Mk2
- Stream Deck Mk2
- Stream Deck Pedal
- Stream Deck Plus
