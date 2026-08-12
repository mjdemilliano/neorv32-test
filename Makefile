.PHONY: all
all: toplevel

sim_build/neorv32:
	mkdir sim_build && \
	nvc --work=/Users/martijn/Development/neorv32-test/sim_build/neorv32 \
	 	-a \
		-f /Users/martijn/Development/neorv32-test/neorv32/rtl/file_list_core.f \
		--preserve-case

.PHONY: toplevel
toplevel: sim_build/neorv32
	nvc --work=sim_build/top \
		-L /Users/martijn/Development/neorv32-test/sim_build \
		-a \
		/Users/martijn/Development/neorv32-test/neorv32/rtl/test_setups/neorv32_test_setup_approm.vhd \
		--preserve-case

.PHONY: run
run:
	nvc --work=top:/Users/martijn/Development/neorv32-test/sim_build/top \
		-L /Users/martijn/Development/neorv32-test/sim_build \
		-e neorv32_test_setup_approm \
		--no-save --jit -r \
		--load=/Users/martijn/Development/neorv32-test/.venv/lib/python3.10/site-packages/cocotb/libs/libcocotbvhpi_nvc.so
