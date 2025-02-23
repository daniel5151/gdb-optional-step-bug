#!/bin/bash

gdb=${GDB:-gdb}

function run_test {
	cargo run ${2:+ -- }${2} 2>&1 1>/dev/null &
	cargo=$!
	${gdb} -ex "set arch ${1}" -x try_stepi.gdb -ex quit &>/dev/null
	kill ${cargo} &>/dev/null
}
export RUST_LOG="error,gdbstub::protocol=trace"

${gdb} --version 2>/dev/null | head -1 | tr -d '\n' && echo -n " for " && ${gdb} --configuration 2>/dev/null | sed -n 's/.*--target=\([^ ]*\).*/\1/p'

for arch in ${@:1}; do
	echo " ===================== [${arch}] ====================="
	run_test ${arch} | grep -aw vCont
	echo
	echo "=================== [${arch} (SS)] ==================="
	run_test ${arch} --single-step | grep -aw vCont
	echo
done

