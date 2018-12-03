.PHONY: all
all:
	@echo 'Please use `cargo` commands to manage building/testing qadapt'

.PHONY: contributors
contributors:
	@echo '`qadapt` is developed by:' > CONTRIBUTORS.md
	@echo '  Bradlee Speice <bradlee@speice.io>' >> CONTRIBUTORS.md
	@git log --format='  %aN <%aE>' | grep -v "Bradlee Speice <bradlee@speice.io>" | sort -u >> CONTRIBUTORS.md

.PHONY: readme README.md
readme: README.md

README.md: src/lib.rs
	@sed -i '/---/q' README.md
	@cat src/lib.rs | grep '//!' | sed 's/^\/\/\! *//g' >> README.md

.PHONY: doc
doc: readme contributors
