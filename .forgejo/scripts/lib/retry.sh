# retry <max> <cmd...>: linear backoff, 10s per attempt.
retry() {
  local max=$1
  shift
  local attempt=1 rc=1
  while (( attempt <= max )); do
    if "$@"; then
      return 0
    else
      rc=$?
    fi
    if (( attempt < max )); then
      echo "attempt $attempt/$max failed (exit $rc); sleeping $((attempt * 10))s" >&2
      sleep $((attempt * 10))
    fi
    (( attempt++ ))
  done
  return "$rc"
}
