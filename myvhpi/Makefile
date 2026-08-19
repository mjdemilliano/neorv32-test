CFLAGS += -I/opt/homebrew/include

myvhpi.so: myvhpi.c
	$(CC) $(CFLAGS) -o $@ $< -bundle -bundle_loader $(shell which nvc)
