use vhpi::{CbData, CbReason, Format, OneToOne, Value, startup_routines};

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

fn start_of_simulation(_data: &CbData) {
    let root = vhpi::handle(OneToOne::RootInst);
    let gpio_out = root.handle_by_name("gpio_out").expect("signal gpio_out not found");
    gpio_out.register_cb(CbReason::ValueChange, gpio_out_changed).expect("failed to register value-change callback");
}

#[unsafe(no_mangle)]
pub extern "C" fn startup() {
    vhpi::printf!("myvhpi plugin loaded");
    vhpi::register_cb(CbReason::StartOfSimulation, start_of_simulation).expect("error registering callback for start of simulation");
}

startup_routines! {
    startup,
}
