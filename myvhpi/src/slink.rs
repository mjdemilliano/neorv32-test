use std::sync::{Arc, Mutex};

use vhpi::{CbData, CbReason, Format, Handle, LogicVal, PutValueMode::Deposit, Value};

pub type Node = u8;

#[derive(Debug)]
pub struct ReceivedMessage<'a> {
    pub source: Node,
    pub data: &'a [u32],
}

pub type MessageHandler = fn(&ReceivedMessage);

#[derive(Debug)]
#[allow(dead_code)]
pub struct Interface {
    // Note: don't put additional things here which are not Sync or Send, because I'm telling below that `Interface` is `Sync` and `Send`
    // to be able to put this whole thing in a `OnceCell`, see below.
    rx_dat_i: Handle,
    rx_src_i: Handle,
    rx_val_i: Handle,
    rx_lst_i: Handle,
    rx_rdy_o: Handle,
    tx_dat_o: Handle,
    tx_dst_o: Handle,
    tx_val_o: Handle,
    tx_lst_o: Handle,
    tx_rdy_i: Handle,

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
        let interface = Arc::new(Interface {
            rx_dat_i: root.handle_by_name("rx_dat_i").expect("signal rx_dat_i not found"),
            rx_src_i: root.handle_by_name("rx_src_i").expect("signal rx_src_i not found"),
            rx_val_i: root.handle_by_name("rx_val_i").expect("signal rx_val_i not found"),
            rx_lst_i: root.handle_by_name("rx_lst_i").expect("signal rx_lst_i not found"),
            rx_rdy_o: root.handle_by_name("rx_rdy_o").expect("signal rx_rdy_o not found"),
            tx_dat_o: root.handle_by_name("tx_dat_o").expect("signal tx_dat_o not found"),
            tx_dst_o: root.handle_by_name("tx_dst_o").expect("signal tx_dst_o not found"),
            tx_val_o: root.handle_by_name("tx_val_o").expect("signal tx_val_o not found"),
            tx_lst_o: root.handle_by_name("tx_lst_o").expect("signal tx_lst_o not found"),
            tx_rdy_i: root.handle_by_name("tx_rdy_i").expect("signal tx_rdy_i not found"),
            on_message_received,
            rx_buffer: Mutex::new(Vec::new()),
        });
        let interface_ref = interface.clone();
        interface.rx_val_i.register_cb(CbReason::ValueChange, move |data| {
            interface_ref.rx_val_i_changed(data)
        }).expect("failed to register value-change callback");
        // Make rx_rdy_o high to enable data being transferred.
        let _ = interface.rx_rdy_o.put_value(Value::Logic(LogicVal::One), Deposit).expect("failed to set rx_rdy_o");
        interface
    }

    fn rx_val_i_changed(&self, data: &CbData) {
        let rx_val_i = match data.obj().get_value(Format::Logic) {
            Ok(Value::Logic(value)) => value,
            Ok(other) => panic!("got unexpected result type for rx_val_i: {other:?}"),
            Err(err) => panic!("failed to read rx_val_i: {err:?}"),
        };
        if rx_val_i == LogicVal::H || rx_val_i == LogicVal::One {
            // Value should be valid.
            let rx_src_i = match data.obj().get_value(Format::ObjType) {
                Ok(Value::LogicVec(bits)) => bits,
                Ok(other) => panic!("got unexpected result type for rx_src_i: {other:?}"),
                Err(err) => panic!("failed to read rx_src_i: {err:?}"),
            };
            let rx_lst_i = match data.obj().get_value(Format::Logic) {
                Ok(Value::Logic(value)) => value,
                Ok(other) => panic!("got unexpected result type for rx_lst_i: {other:?}"),
                Err(err) => panic!("failed to read rx_lst_i: {err:?}"),
            };
            let rx_dat_i = match data.obj().get_value(Format::ObjType) {
                Ok(Value::LogicVec(bits)) => bits,
                Ok(other) => panic!("got unexpected result type for rx_dat_i: {other:?}"),
                Err(err) => panic!("failed to read rx_dat_i: {err:?}"),
            };
            let rx_dat_i_bitstring = rx_dat_i.to_string();
            vhpi::printf!("SLINK RX became valid: value received from {rx_src_i:?}: {rx_dat_i_bitstring} [lst={rx_lst_i:?}]");

            // Add data to buffer.
            let new_word: u32 = match rx_dat_i.try_into() {
                Ok(value) => value,
                Err(err) => panic!("error converting data word to u32: {err:?}"),
            };
            let mut buffer = self.rx_buffer.lock().unwrap();
            buffer.push(new_word);

            // When message is complete, call the handler.
            if rx_lst_i == LogicVal::H || rx_lst_i == LogicVal::One {
                let source: Node = match rx_src_i.try_into() {
                    Ok(value) => value,
                    Err(err) => panic!("error converting src vec into Node: {err:?}"),
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
