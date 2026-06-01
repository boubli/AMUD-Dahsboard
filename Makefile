.PHONY: build clean run test

build:
	CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -ldflags="-s -w" -o amud-bin ./cmd/server

clean:
	rm -f amud-bin
	rm -rf data/

run:
	go run ./cmd/server

test:
	go test ./...
