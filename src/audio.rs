use pulsectl::controllers::DeviceControl;
use pulsectl::controllers::SinkController;

fn play() {
    // create handler that calls functions on playback devices and apps
    let mut handler = SinkController::create().unwrap();

    let devices = handler
        .list_devices()
        .expect("Could not get list of playback devices.");

    println!("Playback Devices: ");
    for dev in devices.clone() {
        println!("{:#?}", dev);
    }
}
