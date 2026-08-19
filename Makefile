.PHONY: all
all: toplevel

.PHONY: hello_world
hello_world:
	(cd neorv32/sw/example/hello_world && \
		make USER_FLAGS+=-DUART0_SIM_MODE clean image install \
	)

.PHONY: blink_led
blink_led:
	(cd neorv32/sw/example/demo_blink_led && \
		make USER_FLAGS+=-DUART0_SIM_MODE clean image install \
	)

# Define here which example program is installed in the imem
# neorv32/rtl/core/neorv32_imem_image.vhd: hello_world
neorv32/rtl/core/neorv32_imem_image.vhd: blink_led

sim_build/neorv32: neorv32/rtl/core/neorv32_imem_image.vhd
	mkdir -p sim_build && \
	nvc --work=sim_build/neorv32 \
	 	-a \
		-f /Users/martijn/Development/neorv32-test/neorv32/rtl/file_list_core.f \
		/Users/martijn/Development/neorv32-test/neorv32/sim/psram_model.vhd \
		--preserve-case

.PHONY: toplevel
toplevel: sim_build/neorv32
	nvc --work=sim_build/work \
		-L /Users/martijn/Development/neorv32-test/sim_build \
		-a \
		--preserve-case \
		/Users/martijn/Development/neorv32-test/neorv32/sim/sim_uart_rx.vhd \
		/Users/martijn/Development/neorv32-test/neorv32/sim/xbus_fmem.vhd \
		/Users/martijn/Development/neorv32-test/neorv32/sim/xbus_gateway.vhd \
		/Users/martijn/Development/neorv32-test/neorv32/sim/xbus_memory.vhd \
		/Users/martijn/Development/neorv32-test/neorv32/sim/jtag_dmi_pkg.vhd \
		/Users/martijn/Development/neorv32-test/neorv32/sim/neorv32_tb.vhd

.PHONY: run
run: toplevel
	nvc --work=sim_build/work \
		-L /Users/martijn/Development/neorv32-test/sim_build \
		-e neorv32_tb \
		-r \
		--ieee-warnings=off \
		neorv32_tb \
		--stop-time=100ms \
		--wave=sim_build/waves.fst


.PHONY: singleshot
singleshot: sim_build/neorv32
	mkdir -p sim_build && \
	nvc --work=neorv32:sim_build/neorv32 \
		-L /Users/martijn/Development/neorv32-test/sim_build \
		-a \
		--preserve-case \
	    -f /Users/martijn/Development/neorv32-test/neorv32/rtl/file_list_core.f \
		/Users/martijn/Development/neorv32-test/neorv32/sim/psram_model.vhd \
		/Users/martijn/Development/neorv32-test/neorv32/sim/sim_uart_rx.vhd \
		/Users/martijn/Development/neorv32-test/neorv32/sim/xbus_fmem.vhd \
		/Users/martijn/Development/neorv32-test/neorv32/sim/xbus_gateway.vhd \
		/Users/martijn/Development/neorv32-test/neorv32/sim/xbus_memory.vhd \
		/Users/martijn/Development/neorv32-test/neorv32/sim/jtag_dmi_pkg.vhd \
		/Users/martijn/Development/neorv32-test/neorv32/sim/neorv32_tb.vhd


# The "single" is to see if we can do this in a single command in an attempt to make it easier to use with cocotb.
.PHONY: runsingle
runsingle: singleshot
	nvc --work=sim_build/neorv32 \
		-L /Users/martijn/Development/neorv32-test/sim_build \
		-e neorv32_tb \
		-r \
		--ieee-warnings=off \
		neorv32_tb \
		--stop-time=100ms \
		--wave=sim_build/waves.fst

.PHONY: clean
clean:
	rm -rf sim_build
	rm -f *.log

VHPI_PLUGIN := myvhpi/target/debug/libmyvhpi.dylib
${VHPI_PLUGIN}:
	(cd myvhpi && cargo build)

.PHONE: mysim
mysim: ${VHPI_PLUGIN}
	nvc --work=sim_build/neorv32 \
		-L /Users/martijn/Development/neorv32-test/sim_build \
		-e neorv32_tb \
		-r \
		--load ${VHPI_PLUGIN} \
		--ieee-warnings=off \
		neorv32_tb \
		--stop-time=100ms \
		--wave=sim_build/waves.fst
