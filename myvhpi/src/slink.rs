use std::sync::{Arc, Mutex};

use vhpi::{CbData, CbReason, Format, Handle, LogicVal, PutValueMode, Value, LogicVec};

pub type Node = u8;

const CLOCK_CYCLE_FS: i64 = 1 * 1000 * 1000 * 1000; // 1 MHz => 1 us = 1000 ns = 1_000_000 ps = 1_000_000_000 fs

#[derive(Debug)]
pub struct ReceivedMessage<'a> {
    pub source: Node,
    pub data: &'a [u32],
}

pub type MessageHandler = fn(&Interface, &ReceivedMessage);
pub type RXReadyHandler = fn(&Interface, bool);

#[derive(Debug)]
pub enum Error {
    RXNotReady,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct OneDirectionInterface {
    // Note: don't put additional things here which are not Sync or Send, because I'm telling below that `Interface` is `Sync` and `Send`
    // to be able to put this whole thing in a `OnceCell`, see below.
    data: Handle,
    addr: Handle,
    valid: Handle,
    last: Handle,
    ready: Handle,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Interface {
    // Note: don't put additional things here which are not Sync or Send, because I'm telling below that `Interface` is `Sync` and `Send`
    // to be able to put this whole thing in a `OnceCell`, see below.
    rx: OneDirectionInterface,
    tx: OneDirectionInterface,
    slink_rx_handle: Handle,
    slink_tx_handle: Handle,

    on_message_received: Option<MessageHandler>,
    on_rx_ready_changed: Option<RXReadyHandler>,
    rx_buffer: Mutex<Vec<u32>>
}

// This is a bit tricky. I need `Interface` to be `Sync` and `Send` in order to be able to put it in a `OnceCell`. Without `Sync` or `Send`
// it complains about `*mut u32` not being `Sync` or `Send` because that is what `Handle` actually is.
// Since we're not using this in a multithreaded context, and the handles are managed by VPHI, let's see if this works.
unsafe impl Sync for Interface {}
unsafe impl Send for Interface {}

impl Interface {
    // Note: need to return `Rc` because need to be able to clone a reference to it for use in the callback handlers.
    // Actually, need to return `Arc` otherwise cannot put it in a OnceLock.
    pub fn init(root: &Handle, on_message_received: Option<MessageHandler>, on_rx_ready_changed: Option<RXReadyHandler>) -> Arc<Interface> {
        let slink_rx = root.handle_by_name("slink_rx").expect("signal slink_rx not found");
        let slink_tx = root.handle_by_name("slink_tx").expect("signal slink_tx not found");
        let slink_rx_interface = OneDirectionInterface {
            data: slink_rx.handle_by_name("data").expect("signal slink_rx.data not found"),
            addr: slink_rx.handle_by_name("addr").expect("signal slink_rx.addr not found"),
            valid: slink_rx.handle_by_name("valid").expect("signal slink_rx.valid not found"),
            last: slink_rx.handle_by_name("last").expect("signal slink_rx.last not found"),
            ready: slink_rx.handle_by_name("ready").expect("signal slink_rx.ready not found"),
        };
        let slink_tx_interface = OneDirectionInterface {
            data: slink_tx.handle_by_name("data").expect("signal slink_tx.data not found"),
            addr: slink_tx.handle_by_name("addr").expect("signal slink_tx.addr not found"),
            valid: slink_tx.handle_by_name("valid").expect("signal slink_tx.valid not found"),
            last: slink_tx.handle_by_name("last").expect("signal slink_tx.last not found"),
            ready: slink_tx.handle_by_name("ready").expect("signal slink_tx.ready not found"),
        };
        let interface: Arc<Interface> = Arc::new(Interface {
            slink_rx_handle: slink_rx,
            slink_tx_handle: slink_tx,
            rx: slink_rx_interface,
            tx: slink_tx_interface,
            on_message_received,
            on_rx_ready_changed,
            rx_buffer: Mutex::new(Vec::new()),
        });
        let interface_ref = interface.clone();
        interface.tx.valid.register_cb(CbReason::ValueChange, move |data| {
            interface_ref.tx_valid_changed(data)
        }).expect("failed to register value-change callback");
        // Make rx.ready high to enable data being transferred.
        let interface_ref2 = interface.clone();
        let _ = interface.rx.ready.put_value(Value::Logic(LogicVal::One), PutValueMode::ForcePropagate).expect("failed to set rx.ready");
        interface.rx.ready.register_cb(CbReason::ValueChange, move |data| {
            interface_ref2.rx_ready_changed(data)
        }).expect("failed to register value-change callback");
        interface
    }

    fn tx_valid_changed(&self, data: &CbData) {
        let tx_valid = match data.obj().get_value(Format::Logic) {
            Ok(Value::Logic(value)) => value,
            Ok(other) => panic!("got unexpected result type for tx.valid: {other:?}"),
            Err(err) => panic!("failed to read tx.valid: {err:?}"),
        };
        if tx_valid == LogicVal::H || tx_valid == LogicVal::One {
            // Value should be valid.
            let tx_source_addr = match self.tx.addr.get_value(Format::ObjType) {
                Ok(Value::LogicVec(bits)) => bits,
                Ok(other) => panic!("got unexpected result type for tx source addr: {other:?}"),
                Err(err) => panic!("failed to read tx source addr: {err:?}"),
            };
            let tx_source_addr: u8 = match tx_source_addr.try_into() {
                Ok(value) => value,
                Err(err) => panic!("error converting bit vector to u8 for addr: {err:?}"),
            };
            let tx_last = match self.tx.last.get_value(Format::Logic) {
                Ok(Value::Logic(value)) => value,
                Ok(other) => panic!("got unexpected result type for tx last: {other:?}"),
                Err(err) => panic!("failed to read tx last: {err:?}"),
            };
            let tx_data = match self.tx.data.get_value(Format::ObjType) {
                Ok(Value::LogicVec(bits)) => bits,
                Ok(other) => panic!("got unexpected result type for tx data: {other:?}"),
                Err(err) => panic!("failed to read tx data: {err:?}"),
            };

            // Add data to buffer.
            let new_word: u32 = match tx_data.try_into() {
                Ok(value) => value,
                Err(err) => panic!("error converting data word to u32: {err:?}"),
            };
            // vhpi::printf!("SLINK TX became valid: value received from {tx_source_addr}: {new_word:x} [lst={tx_last:?}]");

            let mut buffer = self.rx_buffer.lock().unwrap();
            buffer.push(new_word);

            // When message is complete, call the handler.
            if tx_last == LogicVal::H || tx_last == LogicVal::One {
                let source: Node = match tx_source_addr.try_into() {
                    Ok(value) => value,
                    Err(err) => panic!("error converting addr vec into Node: {err:?}"),
                };
                vhpi::printf!("Interface.tx_valid_changed(): last bit was set, so this is the last block, will call handler");
                if let Some(handler) = self.on_message_received {
                    handler(self, &ReceivedMessage { source, data: buffer.as_slice() });
                }
                // Clear the buffer.
                buffer.clear();
            }
        }
        else
        {
            // vhpi::printf!("SLINK TX became invalid");
        }
    }

    pub fn rx_is_ready(&self) -> bool {
        match self.rx.ready.get_value(Format::Logic) {
            Ok(Value::Logic(value)) => {
                value == LogicVal::One
            },
            Ok(other) => panic!("unexpected result type for ready: {other:?}"),
            Err(err) => panic!("error getting value for ready: {err:?}"),
        }
    }

    pub fn rx_ready_changed(&self, data: &CbData) {
        let rx_ready = match data.obj().get_value(Format::Logic) {
            Ok(Value::Logic(value)) => value,
            Ok(other) => panic!("got unexpected result type for rx.ready: {other:?}"),
            Err(err) => panic!("failed to read rx.ready: {err:?}"),
        };
        let rx_is_ready = rx_ready == LogicVal::H || rx_ready == LogicVal::One;
        if let Some(handle_rx_ready) = self.on_rx_ready_changed {
            // Call the handler, but only on the next clock cycle, to avoid oscillations. This is the equivalent of using a register
            // in VHDL instead of using combinatorial logic.
            // We are going to do something dirty here: we are assuming that Interface does not move, so that we can pass a reference
            // to ourselves in the callback. Maybe we should do this using Pin or something but that is for later.
            let me = self as *const Interface;
            vhpi::register_cb_after_delay(CLOCK_CYCLE_FS.into(), move |_| {
                // SAFETY: Assuming that Interface is never moved.
                let interface_ref: &Interface = unsafe { &*me };
                handle_rx_ready(interface_ref, rx_is_ready);
            }).expect("error scheduling callback for next time step");
        }
    }

    pub fn send_word(&self, destination: u8, word: u32, is_last: bool) -> Result<(), Error> {
        let is_ready = self.rx_is_ready();
        vhpi::printf!("Interface.send_word(): sending next word on rx: {word:08x} (last={is_last:?})");
        if !is_ready {
            return Err(Error::RXNotReady);
        }
        self.rx.addr.put_value(Value::LogicVec(LogicVec::from_uint(destination, 4)), PutValueMode::ForcePropagate).expect("failed to put destination addr");
        self.rx.data.put_value(Value::LogicVec(LogicVec::from_uint(word, 32)), PutValueMode::ForcePropagate).expect("failed to put rx data");
        self.rx.last.put_value(Value::Logic(if is_last { LogicVal::One } else { LogicVal::Zero }), PutValueMode::ForcePropagate).expect("failed to set last flag");
        // Only set things valid on the next clock cycle to avoid oscillations.
        let me = self as *const Interface;
        vhpi::register_cb_after_delay(CLOCK_CYCLE_FS.into(), move |_| {
            // SAFETY: Assuming that Interface is never moved.
            let interface_ref: &Interface = unsafe { &*me };
            interface_ref.rx.valid.put_value(Value::Logic(LogicVal::One), PutValueMode::ForcePropagate).expect("failed to set data to be valid");
        }).expect("error scheduling callback for next time step");
        Ok(())
    }

}

impl Drop for Interface {
    fn drop(&mut self) {
        vhpi::printf("slink::Interface dropped");
    }
}
