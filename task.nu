# Define the DOCKER_TAG variable
let DOCKER_TAG = "rcore-docker"

# The `docker` task
def mnt_docker [] {
    let mount_path = $"(pwd | path expand):/mnt"
    ^docker run --rm -it -v $mount_path -w /mnt --name rcore-tutorial-v3 $DOCKER_TAG bash
}

# The `build_docker` task
def build_docker [] {
    ^docker build -t $env.DOCKER_TAG --target build .
}

# The `fmt` task
def fmt [] {
    cd easy-fs
    cargo fmt
    cd ../easy-fs-fuse
    cargo fmt
    cd ../os
    cargo fmt
    cd ../user
    cargo fmt
    cd ..
}