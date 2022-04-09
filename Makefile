docker:
	docker build . -t mona

run:
	docker run --rm -it -v $(CURDIR):/mona -w /mona mona bash

# For M1 Mac
build:
	cargo build --target x86_64-unknown-linux-musl

# They needs to run in the docker container.
test:
	./test.sh

clean:
	rm -f tmp*
