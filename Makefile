docker:
	docker build . -t mona

run:
	docker run --rm -it -v $(CURDIR):/mona -w /mona mona bash

# They needs to run in the docker container.
build:
	cargo build

test: build
	./test.sh

clean:
	rm -f tmp*
