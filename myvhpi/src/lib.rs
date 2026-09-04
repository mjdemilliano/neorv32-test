mod slink;

use std::{collections::VecDeque, sync::{Arc, OnceLock, RwLock}};

use vhpi::{CbData, CbReason, Format, OneToOne, Value, startup_routines};

// Need to have a place for storing our SLINK interface object. It will be initialized when `start_of_simulation`
// is called, so you must be sure to not use it before that routine has run.
static SLINK_INTERFACE: OnceLock<Arc<slink::Interface>> = OnceLock::new();
// Need to have a place for storing the response. Using `RwLock` instead of `RefCell` because `VecDeque<u32>` is not Sync.
static RESPONSE: OnceLock<RwLock<VecDeque<u32>>> = OnceLock::new();

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

fn slink_message_received(slink: &slink::Interface, message: &slink::ReceivedMessage) {
    vhpi::printf!("SLINK message received from {}: {}", message.source, message.data.iter().map(|block| format!("{:08x}", block)).collect::<Vec<_>>().join(" "));
    let lock = RESPONSE.get().expect("expecting RESPONSE mutex to exist");
    let mut response = lock.write().expect("failed to get write lock on RESPONSE");
    if !response.is_empty() {
        vhpi::printf!("warning: response buffer is not empty? bailing");
    } else {
        response.push_back(0x01020304);
        response.push_back(0x09080706);
        if slink.tx_is_ready() {
            let new_word = response.pop_front().expect("failed to pop front (even though I just pushed stuff?)");
            let is_last = response.is_empty();
            slink.send_word(0, new_word, is_last).expect("failed to send next word");
        } else {
            vhpi::printf!("tx is not ready for receiving response");
        }
    }
}

fn slink_tx_ready_changed(slink: &slink::Interface, ready: bool) {
    vhpi::printf!("slink_tx ready changed: {ready:?}");
    let lock = RESPONSE.get().expect("failed to get RESPONSE");
    let mut response = lock.write().expect("failed to get write lock on RESPONSE");
    if ready && !response.is_empty() {
        if let Some(new_word) = response.pop_front() {
            let is_last = response.is_empty();
            slink.send_word(0, new_word, is_last).expect("failed to send next word");
        }
    }
}

fn start_of_simulation(_data: &CbData) {
    RESPONSE.get_or_init(|| RwLock::new(VecDeque::new()));

    let root = vhpi::handle(OneToOne::RootInst);
    let gpio_out = root.handle_by_name("gpio_out").expect("signal gpio_out not found");
    gpio_out.register_cb(CbReason::ValueChange, gpio_out_changed).expect("failed to register value-change callback");

    let slink = slink::Interface::init(&root, Some(slink_message_received),Some(slink_tx_ready_changed));
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
