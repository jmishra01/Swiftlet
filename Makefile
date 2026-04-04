PYTHON := ./.venv/bin/python
PIP := ./.venv/bin/pip
MATURIN := ./.venv/bin/maturin
WHEEL_DIR := target/wheels
PYTHONPATH_LOCAL := pyswiftlet/python
PY_EXTENSION_GLOB := pyswiftlet/python/swiftlet/*.so
ROUNDS ?= 200
REPETITIONS ?= 50
ALGORITHM ?= earley

.PHONY: build-rust build-python build-python-extension test-python bench-rust bench-python compare-benchmarks clean-python-artifacts

build-rust:
	cargo build -p swiftlet --release

test-rust:
	cargo test -p swiftlet -q

bench-rust:
	cargo bench -p swiftlet -q

coverage-rust:
	cargo llvm-cov

build-python:
	mkdir -p $(WHEEL_DIR)
	env -u VIRTUAL_ENV PYO3_PYTHON=$(abspath $(PYTHON)) $(MATURIN) build --release -m pyswiftlet/Cargo.toml -o $(WHEEL_DIR)
	$(PIP) install --force-reinstall $$(ls -t $(WHEEL_DIR)/swiftlet-*.whl | head -n 1)

build-python-extension:
	env -u VIRTUAL_ENV PYO3_PYTHON=$(abspath $(PYTHON)) $(MATURIN) develop --release -m pyswiftlet/Cargo.toml

test-python:
	@set -e; \
	trap '$(MAKE) clean-python-artifacts' EXIT; \
	$(MAKE) build-python-extension; \
	PYTHONPATH=$(PYTHONPATH_LOCAL) $(PYTHON) -m unittest discover -s pyswiftlet/tests -v

clean-python-artifacts:
	find pyswiftlet -type d -name __pycache__ -prune -exec rm -rf {} +
	rm -rf pyswiftlet/python/swiftlet/libpyswiftlet*
	rm -f $(PY_EXTENSION_GLOB)
