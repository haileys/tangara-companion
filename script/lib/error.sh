handle-error() {
    die "command failed: ${BASH_COMMAND}"
}

trap handle-error ERR
