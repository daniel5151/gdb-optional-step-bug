#!/bin/bash

gdb=${GDB:-gdb}

function run_test {
	cargo run --features "stub_${1}"${2:+ -- }${2} 2>&1 1>/dev/null &
	cargo=$!
	${gdb} -x try_stepi.gdb -ex quit &>/dev/null
	kill ${cargo} &>/dev/null
}
declare -A supported_archs=([arm]= [mips]= [x86]=)

export RUST_LOG="error,gdbstub::protocol=trace"

${gdb} --version | head -n 1

for arch in ${@:1}; do
	if [[ -v supported_archs[$arch] ]]; then
		echo " ===================== [${arch}] ====================="
		run_test ${arch} | grep -aw vCont
		echo
		echo "=================== [${arch} (SS)] ==================="
		run_test ${arch} --single-step | grep -aw vCont
		echo
	else
		echo "Unknown architecture: ${arch}" 1>&2
	fi
done

