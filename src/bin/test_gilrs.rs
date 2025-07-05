use gilrs::{Gilrs, Event};

fn main() {
    let mut gilrs = Gilrs::new().unwrap();

    println!("Waiting for gamepad events...");
    println!("Press buttons or move sticks on your controller.");
    
    // List connected gamepads
    for (id, gamepad) in gilrs.gamepads() {
        println!("{:?} is {:?} ({})", id, gamepad.power_info(), gamepad.name());
    }
    
    loop {
        while let Some(Event { id, event, time: _ }) = gilrs.next_event() {
            println!("{:?}: {:?}", id, event);
        }
    }
}