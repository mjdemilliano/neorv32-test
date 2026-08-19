CFLAGS += -I/opt/homebrew/include -Wall -Werror

myvhpi.so: myvhpi.c
	$(CC) $(CFLAGS) -o $@ $< -bundle -bundle_loader $(shell which nvc)
