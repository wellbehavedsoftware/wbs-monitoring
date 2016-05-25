default: all

all: libaptc.a

libaptc.o: libaptc.cc
	g++ -c -o $@ $< -fPIC

libaptc.a: libaptc.o
	ar rcs $@ $^

clean:
	rm -f *.o libaptc.a
