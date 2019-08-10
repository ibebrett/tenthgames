CC=g++
LDFLAGS =-lraylib -lGL -lm -lpthread -ldl -lrt -lX11

%.o: %.cc
	$(CC) -c -o $@ $<

hello: hello.o
	$(CC) -o hello hello.o $(LDFLAGS)
