PYTHON := ./.venv/bin/python
PIP := ./.venv/bin/pip
MATURIN := ./.venv/bin/maturin
WHEEL_DIR := target/wheels
PYTHONPATH_LOCAL := pyswiftlet/python
ROUNDS ?= 200
REPETITIONS ?= 50
ALGORITHM ?= earley

.PHONY: build-rust build-python test-python bench-rust bench-python compare-benchmarks clean-python-artifacts

build-rust:
	cargo build -p swiftlet --release

build-python:
	mkdir -p $(WHEEL_DIR)
	env -u VIRTUAL_ENV PYO3_PYTHON=$(abspath $(PYTHON)) $(MATURIN) build --release -m pyswiftlet/Cargo.toml -o $(WHEEL_DIR)
	$(PIP) install --force-reinstall $$(ls -t $(WHEEL_DIR)/swiftlet-*.whl | head -n 1)

test-python:
	PYTHONPATH=$(PYTHONPATH_LOCAL) $(PYTHON) -m unittest discover -s pyswiftlet/tests -v

bench-rust: build-rust
	cargo run --release -p swiftlet --example benchmark_parse -- --rounds $(ROUNDS) --repetitions $(REPETITIONS) --algorithm $(ALGORITHM)

bench-python: build-python
	$(PYTHON) pyswiftlet/benchmarks/benchmark_parse.py --rounds $(ROUNDS) --repetitions $(REPETITIONS) --algorithm $(ALGORITHM)

compare-benchmarks: build-rust build-python
	$(PYTHON) pyswiftlet/benchmarks/compare_benchmarks.py --rounds $(ROUNDS) --repetitions $(REPETITIONS) --algorithm $(ALGORITHM)

clean-python-artifacts:
	find pyswiftlet -type d -name __pycache__ -prune -exec rm -rf {} +
	rm -rf pyswiftlet/python/swiftlet/libpyswiftlet*
	rm pyswiftlet/python/swiftlet/*.so
