docker_image := "reqlang-expr:0.7.0"

# Build docker image for reqlang cli
build-docker:
    docker build -t {{docker_image}} .

build-docker-no-cache:
    docker build --no-cache -t {{docker_image}} .

# Run docker image for reqlang cli
run-docker *cli_args:
    docker run -it --rm --read-only {{docker_image}} {{cli_args}}