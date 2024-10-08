# Variables
RJEM_MALLOC_CONF=prof_leak:true,lg_prof_sample:0,prof_final:true,prof_prefix:./profile/jeprof

# Targets
.PHONY: run run_release clean prof

run:
	_RJEM_MALLOC_CONF=$(RJEM_MALLOC_CONF) cargo run
	jeprof --svg --show_bytes ./target/debug/dm ./profile/`ls ./profile | tail -n1` > profile.svg

run_release:
	_RJEM_MALLOC_CONF=$(RJEM_MALLOC_CONF) cargo run --release
	jeprof --svg --show_bytes ./target/release/dm ./profile/`ls ./profile | tail -n1` > profile.svg

clean:
	cargo clean
	rm -rf profile
	rm profile.svg
