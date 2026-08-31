//! Cop fixture helpers (nitrocop-compatible `^` annotations).

mod assert;
mod parse;
mod run;

pub use assert::{
    assert_cop_no_offenses_full, assert_cop_no_offenses_full_with_config,
    assert_cop_offenses_full, assert_cop_offenses_full_with_config,
};
pub use parse::{parse_fixture, ExpectedOffense, ParsedFixture};
pub use run::{run_cop_full, run_cop_full_internal, run_cop_full_with_config};

/// `offense.rb` + `no_offense.rb` under `tests/fixtures/<path>/`.
#[macro_export]
macro_rules! cop_fixture_tests {
    ($cop:expr, $path:literal) => {
        #[test]
        fn offense_fixture() {
            $crate::testutil::assert_cop_offenses_full(
                &$cop,
                include_bytes!(concat!("../../../tests/fixtures/", $path, "/offense.rb")),
            );
        }

        #[test]
        fn no_offense_fixture() {
            $crate::testutil::assert_cop_no_offenses_full(
                &$cop,
                include_bytes!(concat!("../../../tests/fixtures/", $path, "/no_offense.rb")),
            );
        }
    };
}
