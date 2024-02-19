CLIPPY_ARGS=-- -D clippy::all -D clippy::pedantic -D clippy::nursery \
	-D missing_docs \
	-D clippy::undocumented_unsafe_blocks \
	-W clippy::needless-pass-by-value \
	-A clippy::missing_const_for_fn \
	-A clippy::type_complexity \
	-A clippy::module_name_repetitions \
	-A clippy::redundant_pub_crate

run:
	cargo run --example simple_scene

check:
	cargo fmt --all -- --check
	cargo clippy --examples --no-default-features $(CLIPPY_ARGS)
	cargo clippy --examples $(CLIPPY_ARGS)
	RUSTDOCFLAGS="-D warnings" cargo doc --examples --no-deps
	cargo test --examples -j12
	cargo test --doc -j12
