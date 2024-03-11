.PHONY: all debug release release_x86 container clean

#ECR_REGISTRY = null
PREFIX = secrets-transfer
#REDIS_IMAGE = $(ECR_REGISTRY)-redis:latest
#SERVER_IMAGE = $(ECR_REGISTRY)-server:latest
REDIS_IMAGE = $(PREFIX)-redis:latest
SERVER_IMAGE = $(PREFIX)-server:latest

# Default target
all: debug release release_x86

debug: compile_debug container

release: compile_release container

release_x86: compile_x86_64 container_x86_64



# Cross-compilation release
compile_x86_64:
	rustup target add x86_64-unknown-linux-gnu
	cargo build --release --target=x86_64-unknown-linux-gnu

container_x86_64: 
    nerdctl build --platform=linux/amd64 -t $(REDIS_IMAGE) ./redis/.
    nerdctl build --platform=linux/amd64 -t $(SERVER_IMAGE) ./src/.



# Current OS target
compile_debug:
	cd src && cargo build
	cp ./target/debug/SecretsTransfer ./src/SecretsTransfer

compile_release:
	cd src && cargo build --release
	cp ./target/release/SecretsTransfer ./src/SecretsTransfer

container: 
	nerdctl build -t $(REDIS_IMAGE) ./redis/.
	nerdctl build -t $(SERVER_IMAGE) ./src/.



# Deploy
deploy: 
	kubectl --kubeconfig $(KUBECONFIG) apply -f deployment.yml

undeploy:
	kubectl --kubeconfig $(KUBECONFIG) delete -f deployment.yml


# Run
run:
	nerdctl run -d -p 6379:6379 --name redis $(REDIS_IMAGE)
#	nerdctl run -d -p 8080:8080 --name server $(SERVER_IMAGE)

# Stop
stop:
	nerdctl stop $(REDIS_IMAGE)
#	nerdctl stop $(SERVER_IMAGE)

# Clean up
clean:
	rm ./src/SecretsTransfer
	cargo clean
