use gix_merge::blob::builtin_driver::binary::{Pick, ResolveWith};
use gix_merge::blob::{builtin_driver, Resolution};

#[test]
fn binary() {
    assert_eq!(
        builtin_driver::binary(None),
        (Pick::Ours, Resolution::Conflict),
        "by default it picks ours and marks it as conflict"
    );
    assert_eq!(
        builtin_driver::binary(Some(ResolveWith::Ancestor)),
        (Pick::Ancestor, Resolution::Complete),
        "Otherwise we can pick anything and it will mark it as complete"
    );
    assert_eq!(
        builtin_driver::binary(Some(ResolveWith::Ours)),
        (Pick::Ours, Resolution::Complete)
    );
    assert_eq!(
        builtin_driver::binary(Some(ResolveWith::Theirs)),
        (Pick::Theirs, Resolution::Complete)
    );
}

mod text {
    use bstr::ByteSlice;
    use gix_merge::blob::Resolution;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn run_baseline() -> crate::Result {
        let root = gix_testtools::scripted_fixture_read_only("text-baseline.sh")?;
        let cases = std::fs::read_to_string(root.join("baseline.cases"))?;
        let mut out = Vec::new();
        for case in baseline::Expectations::new(&root, &cases) {
            let mut input = imara_diff::intern::InternedInput::default();
            dbg!(&case.name, case.options);
            let actual = gix_merge::blob::builtin_driver::text(
                &mut out,
                &mut input,
                &case.ours,
                Some(case.ours_marker.as_str().as_ref()),
                &case.base,
                Some(case.base_marker.as_str().as_ref()),
                &case.theirs,
                Some(case.theirs_marker.as_str().as_ref()),
                case.options,
            );
            let expected_resolution = if case.expected.contains_str("<<<<<<<") {
                Resolution::Conflict
            } else {
                Resolution::Complete
            };
            assert_eq!(actual, expected_resolution, "{}: resolution mismatch", case.name,);
            assert_str_eq!(
                out.as_bstr().to_str_lossy(),
                case.expected.to_str_lossy(),
                "{}: output mismatch\n{}",
                case.name,
                out.as_bstr()
            );
        }
        Ok(())
    }

    mod baseline {
        use bstr::BString;
        use gix_merge::blob::builtin_driver::text::{ConflictStyle, ResolveWith};
        use std::path::Path;

        #[derive(Debug)]
        pub struct Expectation {
            pub ours: BString,
            pub ours_marker: String,
            pub theirs: BString,
            pub theirs_marker: String,
            pub base: BString,
            pub base_marker: String,
            pub name: BString,
            pub expected: BString,
            pub options: gix_merge::blob::builtin_driver::text::Options,
        }

        pub struct Expectations<'a> {
            root: &'a Path,
            lines: std::str::Lines<'a>,
        }

        impl<'a> Expectations<'a> {
            pub fn new(root: &'a Path, cases: &'a str) -> Self {
                Expectations {
                    root,
                    lines: cases.lines(),
                }
            }
        }

        impl Iterator for Expectations<'_> {
            type Item = Expectation;

            fn next(&mut self) -> Option<Self::Item> {
                let line = self.lines.next()?;
                let mut words = line.split(' ');
                let (Some(ours), Some(base), Some(theirs), Some(output)) =
                    (words.next(), words.next(), words.next(), words.next())
                else {
                    panic!("need at least the input and output")
                };

                let read = |rela_path: &str| read_blob(self.root, rela_path);

                let mut options = gix_merge::blob::builtin_driver::text::Options::default();
                for arg in words {
                    match arg {
                        "--diff3" => options.conflict_style = ConflictStyle::Diff3,
                        "--zdiff3" => options.conflict_style = ConflictStyle::ZealousDiff3,
                        "--ours" => options.on_conflict = Some(ResolveWith::Ours),
                        "--theirs" => options.on_conflict = Some(ResolveWith::Theirs),
                        "--union" => options.on_conflict = Some(ResolveWith::Union),
                        _ => panic!("Unknown argument to parse into options: '{arg}'"),
                    }
                }

                Some(Expectation {
                    ours: read(ours),
                    ours_marker: ours.into(),
                    theirs: read(theirs),
                    theirs_marker: theirs.into(),
                    base: read(base),
                    base_marker: base.into(),
                    expected: read(output),
                    name: output.into(),
                    options,
                })
            }
        }

        fn read_blob(root: &Path, rela_path: &str) -> BString {
            std::fs::read(root.join(rela_path))
                .unwrap_or_else(|_| panic!("Failed to read '{rela_path}' in '{}'", root.display()))
                .into()
        }
    }
}
