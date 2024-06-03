PACKAGE_NAME=zsb

ifeq ($(OS),Windows_NT)
	BINARY_OUTPUT := ${PACKAGE_NAME}.lib
else
	BINARY_OUTPUT := lib${PACKAGE_NAME}.a
endif

build: export CGO_ENABLED = 1
build:
	@go mod tidy
	@go build -buildmode=c-archive -o out/${BINARY_OUTPUT} -ldflags "-s -w"

clean:
	@go clean
	-@rm -r out/
