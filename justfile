docker_image := "reqlang-expr:0.8.0"

# List commands
default:
    just --list

# Run tests
test:
    cargo test

# Run tests with coverage enabled
coverage:
    ./coverage.sh

# Run the repl
repl *args:
    ./repl.sh {{args}}

# Build repl docker image 
build-docker:
    docker build -t {{docker_image}} .

# Run repl in built docker image
run-docker *cli_args:
    docker run -it --rm --read-only {{docker_image}} {{cli_args}}