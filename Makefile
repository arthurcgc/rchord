.PHONY: servers
servers:
	cargo run --bin servers

.PHONY: test1
test1:
	curl localhost:8001/0

.PHONY: test2
test2:
	curl localhost:8004/hello

.PHONY: test3
test3:
	curl -X POST "localhost:8004/key1/value1"
