mod slink;

use std::sync::{Arc, OnceLock};

use vhpi::{CbData, CbReason, Format, OneToOne, Value, startup_routines};

// Need to have a place for storing our SLINK interface object. It will be initialized when `start_of_simulation`
// is called, so you must be sure to not use it before that routine has run.
static SLINK_INTERFACE: OnceLock<Arc<slink::Interface>> = OnceLock::new();

fn gpio_out_changed(data: &CbData) {
    vhpi::printf("gpio_out changed!");
    let bits = match data.obj().get_value(Format::ObjType) {
        Ok(Value::LogicVec(bits)) => bits,
        Ok(other) => panic!("gpio_name expected LogicVec, got {other:?}"),
        Err(err) => panic!("failed to read gpio_out: {err}"),
    };
    let bit_string = bits.to_string();
    let bit31 = bits.as_slice()[31];
    vhpi::printf!(" => {bit_string} [31]={bit31}");
}

fn slink_message_received(message: &slink::ReceivedMessage) {
    vhpi::printf!("SLINK message received from {}: {:?}", message.source, message.data);
}

fn start_of_simulation(_data: &CbData) {
    let root = vhpi::handle(OneToOne::RootInst);
    let gpio_out = root.handle_by_name("gpio_out").expect("signal gpio_out not found");
    gpio_out.register_cb(CbReason::ValueChange, gpio_out_changed).expect("failed to register value-change callback");
    let slink = slink::Interface::init(&root, Some(slink_message_received));
    // This is the starting point of this Rust code, so we are sure that `SLINK_INTERFACE` hasn't yet been initialized.
    // So it's ok to do this.
    let _ = SLINK_INTERFACE.set(slink).expect("failed to set SLINK_INTERFACE global");
}

#[unsafe(no_mangle)]
pub extern "C" fn startup() {
    vhpi::printf!("myvhpi plugin loaded");
    vhpi::register_cb(CbReason::StartOfSimulation, start_of_simulation).expect("error registering callback for start of simulation");
}

startup_routines! {
    startup,
}
