use std::sync::{Arc, Mutex};

use vhpi::{CbData, CbReason, Format, Handle, LogicVal, PutValueMode::ForcePropagate, Value};

pub type Node = u8;

#[derive(Debug)]
pub struct ReceivedMessage<'a> {
    pub source: Node,
    pub data: &'a [u32],
}

pub type MessageHandler = fn(&ReceivedMessage);

#[derive(Debug)]
#[allow(dead_code)]
struct OneDirectionInterface {
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
    pub fn init(root: &Handle, on_message_received: Option<MessageHandler>) -> Arc<Interface> {
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
            rx_buffer: Mutex::new(Vec::new()),
        });
        let interface_ref = interface.clone();
        interface.rx.valid.register_cb(CbReason::ValueChange, move |data| {
            interface_ref.rx_valid_changed(data)
        }).expect("failed to register value-change callback");
        // Make rx.ready high to enable data being transferred.
        let _ = interface.rx.ready.put_value(Value::Logic(LogicVal::One), ForcePropagate).expect("failed to set rx.ready");
        interface
    }

    fn rx_valid_changed(&self, data: &CbData) {
        let rx_valid = match data.obj().get_value(Format::Logic) {
            Ok(Value::Logic(value)) => value,
            Ok(other) => panic!("got unexpected result type for rx.valid: {other:?}"),
            Err(err) => panic!("failed to read rx.valid: {err:?}"),
        };
        if rx_valid == LogicVal::H || rx_valid == LogicVal::One {
            // Value should be valid.
            let rx_source_addr = match self.rx.addr.get_value(Format::ObjType) {
                Ok(Value::LogicVec(bits)) => bits,
                Ok(other) => panic!("got unexpected result type for rx source addr: {other:?}"),
                Err(err) => panic!("failed to read rx source addr: {err:?}"),
            };
            let rx_source_addr: u8 = match rx_source_addr.try_into() {
                Ok(value) => value,
                Err(err) => panic!("error converting bit vector to u8 for addr: {err:?}"),
            };
            let rx_last = match self.rx.last.get_value(Format::Logic) {
                Ok(Value::Logic(value)) => value,
                Ok(other) => panic!("got unexpected result type for rx last: {other:?}"),
                Err(err) => panic!("failed to read rx last: {err:?}"),
            };
            let rx_data = match self.rx.data.get_value(Format::ObjType) {
                Ok(Value::LogicVec(bits)) => bits,
                Ok(other) => panic!("got unexpected result type for rx data: {other:?}"),
                Err(err) => panic!("failed to read rx data: {err:?}"),
            };
            let rx_data_bitstring = rx_data.to_string();
            vhpi::printf!("SLINK RX became valid: value received from {rx_source_addr}: {rx_data_bitstring} [lst={rx_last:?}]");

            // Add data to buffer.
            let new_word: u32 = match rx_data.try_into() {
                Ok(value) => value,
                Err(err) => panic!("error converting data word to u32: {err:?}"),
            };
            let mut buffer = self.rx_buffer.lock().unwrap();
            buffer.push(new_word);

            // When message is complete, call the handler.
            if rx_last == LogicVal::H || rx_last == LogicVal::One {
                let source: Node = match rx_source_addr.try_into() {
                    Ok(value) => value,
                    Err(err) => panic!("error converting addr vec into Node: {err:?}"),
                };
                if let Some(handler) = self.on_message_received {
                    handler(&ReceivedMessage { source, data: buffer.as_slice() });
                }
            }
        }
        else
        {
            vhpi::printf!("SLINK RX became invalid");
        }
    }
}

impl Drop for Interface {
    fn drop(&mut self) {
        vhpi::printf("slink::Interface dropped");
    }
}
