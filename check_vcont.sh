#!/bin/bash

gdb=${GDB:-gdb}

function run_test {
	cargo run ${2:+ -- }${2} 2>&1 1>/dev/null &
	cargo=$!
	${gdb} -ex "set arch ${1}" -x try_stepi.gdb -ex quit &>/dev/null
	kill ${cargo} &>/dev/null
}
export RUST_LOG="error,gdbstub::protocol=trace"

${gdb} --version | head -n 1

for arch in ${@:1}; do
	echo " ===================== [${arch}] ====================="
	run_test ${arch} | grep -aw vCont
	echo
	echo "=================== [${arch} (SS)] ==================="
	run_test ${arch} --single-step | grep -aw vCont
	echo
done

