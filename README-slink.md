# SLINK

https://stnolting.github.io/neorv32/#_stream_link_interface_slink
https://en.wikipedia.org/wiki/Advanced_eXtensible_Interface

## Mapping between SLINK and AXI-Stream

|AXI-Stream |Source     |SLINK    |Width |Description|
|-----------|-----------|---------|------|-----------|
|ACLK       |           |         | 1    |
|ARESETn    |           |         | 1    |
|TVALID     |Transmitter| val     | 1    | Indicates that a Transmitter is driving a valid transfer. Data transfer takes place if both TVALID and TREADY are asserted. |
|TREADY     |Receiver   | rdy     | 1    | Indicates that a receiver can accept a transfer. |
|TDATA      |Transmitter| dat     | 32   | Data passed across the interface. |
|TLAST      |Transmitter| lst     | 1    | Indicates the boundary of a packet. |
|TID        |Transmitter|         |
|TDEST      |Transmitter| src/dst | 4    | Routing information. |
|TUSER      |Transmitter|
|TWAKEKUP   |Transmitter|
