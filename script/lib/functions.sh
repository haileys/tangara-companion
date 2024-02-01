usage() {
    sed -nEe 's/^###$//p; s/^### (.*)/\1/p' "$0" >&2
    exit 1
}

# nerf colours if tput doesn't exist or stdout is not a tty
COLORS=1
command -v tput >/dev/null && [ -t 0 ] || COLORS=

# useful helper functions
tput() {
    [ -n "$COLORS" ] && command tput "$@"
}

die() {
    echo "$(tput setaf 1 bold)error:$(tput sgr0)$(tput bold)" "$@" "$(tput sgr0)" >&2
    exit 1
}

info() {
    echo "$(tput setaf 4 bold)++++$(tput sgr0)$(tput bold)" "$@" "$(tput sgr0)"
}

win() {
    echo "$(tput setaf 2 bold)yay!$(tput sgr0)$(tput bold)" "$@" "$(tput sgr0)"
}

check-command() {
    local bin="$1"
    local hint="${2:-}"

    [ -n "$hint" ] && hint=", $hint"

    command -v "$bin" >/dev/null || die "command '${bin}' not found in PATH${hint}"
}

log-command() {
    info "running:" "$@"
    "$@"
}
