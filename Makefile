.PHONY: all
all: toplevel

sim_build/neorv32:
	mkdir sim_build && \
	nvc --work=/Users/martijn/Development/neorv32-test/sim_build/neorv32 \
	 	--map nvc:/Users/martijn/Software/nvc-1.17.2/usr/local/lib/nvc/nvc.08 \
		--map std:/Users/martijn/Software/nvc-1.17.2/usr/local/lib/nvc/std.08 \
		--map ieee:/Users/martijn/Software/nvc-1.17.2/usr/local/lib/nvc/ieee.08 \
		-a \
		-f /Users/martijn/Development/neorv32-test/neorv32/rtl/file_list_core.f \
		--preserve-case


.PHONY: toplevel
toplevel: sim_build/neorv32
	nvc --work=sim_build/top \
		-L /Users/martijn/Development/neorv32-test/sim_build \
		--map std:/Users/martijn/Software/nvc-1.17.2/usr/local/lib/nvc/std.08 \
		--map ieee:/Users/martijn/Software/nvc-1.17.2/usr/local/lib/nvc/ieee.08 \
		-a \
		/Users/martijn/Development/neorv32-test/neorv32/rtl/test_setups/neorv32_test_setup_approm.vhd \
		--preserve-case
