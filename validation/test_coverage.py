#!/usr/bin/env python3
"""Focused coverage-scope regressions, no Rust suite or financial fixtures."""
import importlib.util
import pathlib
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location(
    'sabiquant_coverage', pathlib.Path(__file__).with_name('coverage.py'))
COVERAGE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(COVERAGE)


class ProductionScopeTests(unittest.TestCase):
    def test_early_helper_preserves_later_production(self):
        source = 'fn before() {}\n#[cfg(test)]\nfn helper() {}\nfn after() {}\n'
        self.assertEqual(COVERAGE.test_only_ranges(source), [(2, 1, 3, 15)])

    def test_byte_columns_and_half_open_same_line_boundaries(self):
        source = 'fn café() {} #[cfg(test)] fn helper() {} fn after() {}\n'
        first = len(source[:source.index('#')].encode()) + 1
        end = len(source[:source.index(' fn after')].encode()) + 1
        self.assertEqual(COVERAGE.test_only_ranges(source), [(1, first, 1, end)])
        with tempfile.TemporaryDirectory() as directory:
            root, path = self.file(directory, source)
            report, _ = COVERAGE.production_regions(self.export(path, [
                self.region(1, first, 1), self.region(1, end - 1, 1),
                self.region(1, end, 0), self.region(1, end + 1, 1),
                self.region(1, 1, 1)]), root)
            self.assertEqual(report['src/sample.rs']['regions'], 3)
            self.assertEqual(report['src/sample.rs']['covered'], 2)

    def test_nested_modules_comments_and_literals_are_opaque(self):
        source = '''#[cfg(test)]
mod tests {
 mod nested { fn helper<'a>(s: &'a str) {
 let _ = "} \\" #[cfg(test)] {";
 let _ = r###" } " # { "###;
 let _ = br#" } { "#;
 let _ = cr#" } { "#;
 let _ = '\\u{7d}'; let _ = b'}';
 /* } /* { */ } */ // }
 if s.is_empty() { helper("x"); }
 } }
}
fn after() {}
'''
        spans = COVERAGE.test_only_ranges(source)
        self.assertEqual(spans, [(1, 1, 12, 2)])

    def test_test_only_impl(self):
        source = '#[cfg(test)]\nimpl<T: Copy> Foo<T> { fn helper() {} }\nfn after() {}'
        spans = COVERAGE.test_only_ranges(source)
        self.assertEqual(spans[0][:2], (1, 1))
        self.assertEqual(spans[0][2:], (2, 40))

    def test_visibility_signature_groups_and_lifetime(self):
        source = "#[cfg(test)]\npub(in crate::x) fn helper<'a>(s: &'a [u8; 4]) -> Option<u8> { None }\n"
        self.assertEqual(len(COVERAGE.test_only_ranges(source)), 1)

    def test_direct_test_fn_is_excluded(self):
        self.assertEqual(COVERAGE.test_only_ranges('#[test]\nfn unit() {}\nfn after() {}'),
                         [(1, 1, 2, 13)])

    def test_fake_attributes_in_literals_or_comments(self):
        self.assertEqual(COVERAGE.test_only_ranges(
            '// #[cfg(test)]\n/* #[test] */\nconst S: &str = r#"#[cfg(test)]"#;'), [])

    def test_only_conditional_lint_attributes_are_allowed(self):
        for prefix in ['#', '#!']:
            source = prefix + '[cfg_attr(not(test), expect(dead_code, reason="x"), allow(unused))]\nfn after() {}'
            self.assertEqual(COVERAGE.test_only_ranges(source), [])
        for attribute in ['derive(Debug)', 'path="other.rs"', 'cfg(test)',
                          'cfg_attr(test, allow(dead_code))', 'doc="x"']:
            with self.subTest(attribute=attribute), self.assertRaises(ValueError):
                COVERAGE.test_only_ranges('#[cfg_attr(test, '+attribute+')]\nfn after() {}')

    def test_unsupported_cfg_expressions_and_inner_test_gates_fail(self):
        for source in ['#![cfg(test)]\nfn f() {}', '#![test]\nfn f() {}',
                       '#[cfg(not(test))]\nfn f() {}',
                       '#[cfg(any(test,feature="x"))]\nfn f() {}']:
            with self.subTest(source=source), self.assertRaises(ValueError):
                COVERAGE.test_only_ranges(source)

    def test_ambiguous_const_generic_signatures_fail(self):
        for source in ['#[cfg(test)]\nfn helper() -> Foo<{ 1 }> { Foo }',
                       '#[cfg(test)]\nimpl Trait for Foo<{ 1 }> { fn f() {} }']:
            with self.subTest(source=source), self.assertRaises(ValueError):
                COVERAGE.test_only_ranges(source)

    def test_gated_macro_and_macro_produced_gates_fail(self):
        for source in ['#[cfg(test)]\nmake_tests! {}',
                       'macro_rules! make { () => { #[test] fn f() {} } }',
                       'macro_rules! r#make { () => { #[test] fn f() {} } }',
                       'make! { #[cfg(test)] fn f() {} }']:
            with self.subTest(source=source), self.assertRaises(ValueError):
                COVERAGE.test_only_ranges(source)

    def test_raw_builtin_attribute_names_keep_test_semantics(self):
        for attribute in ['r#cfg(test)', 'r#test']:
            source='#['+attribute+'] fn helper() {} fn after() {}'
            spans=COVERAGE.test_only_ranges(source)
            self.assertEqual(len(spans),1)
            self.assertEqual(spans[0][3],source.index(' fn after')+1)

    def test_symlink_target_and_ancestor_are_rejected_before_resolution(self):
        with tempfile.TemporaryDirectory() as directory:
            root=pathlib.Path(directory).resolve();(root/'src').mkdir();(root/'tests/real').mkdir(parents=True)
            (root/'tests/real/support.rs').write_text('pub fn helper() {}')
            (root/'tests/link.rs').symlink_to(root/'tests/real/support.rs')
            (root/'tests/alias').symlink_to(root/'tests/real',target_is_directory=True)
            file=root/'src/lib.rs'
            for path in ['../tests/link.rs','../tests/alias/support.rs']:
                source='#[cfg(test)] #[path="'+path+'"] mod support;'
                with self.subTest(path=path),self.assertRaises(ValueError):
                    COVERAGE.test_only_ranges(source,file,root)

    def test_source_file_and_directory_symlinks_fail_preflight(self):
        with tempfile.TemporaryDirectory() as directory:
            root=pathlib.Path(directory).resolve();(root/'src').mkdir();(root/'other').mkdir()
            target=root/'other/module.rs';target.write_text('fn after() {}')
            for name,destination in [('link.rs',target),('alias',root/'other')]:
                link=root/'src'/name;link.symlink_to(destination,target_is_directory=destination.is_dir())
                with self.subTest(name=name),self.assertRaises(ValueError):
                    COVERAGE.production_regions({'data':[]},root)
                link.unlink()

    def test_unterminated_or_mismatched_tokens_fail(self):
        for source in ['#[cfg(test)]', '#[cfg(test)] fn f() {', '/* unmatched',
                       'const S: &str = r#"unmatched;', 'const S: &str = "unmatched;',
                       '\ufefffn f() {}',
                       '#[cfg(test)] fn f(a: [u8; 1)) {}']:
            with self.subTest(source=source), self.assertRaises(ValueError):
                COVERAGE.test_only_ranges(source)

    def test_unsupported_item_modifiers_or_kinds_fail(self):
        for item in ['const X: u8 = 1;', 'async fn f() {}', 'const fn f() {}',
                     'unsafe fn f() {}', 'use crate::x;']:
            with self.subTest(item=item), self.assertRaises(ValueError):
                COVERAGE.test_only_ranges('#[cfg(test)]\n'+item)

    def test_external_module_resolves_explicit_outside_src_path(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory).resolve()
            (root/'src').mkdir(); (root/'tests').mkdir()
            (root/'tests/support.rs').write_text('pub fn helper() {}')
            path = root/'src/lib.rs'
            for literal in ['"../tests/support.rs"', 'r#"../tests/support.rs"#']:
                source = '#[path='+literal+']\n#[cfg(test)]\nmod support;\nfn after() {}'
                path.write_text(source)
                self.assertEqual(COVERAGE.test_only_ranges(source,path,root), [(1,1,3,13)])

    def test_external_src_module_fails_even_if_parent_has_no_llvm_regions(self):
        with tempfile.TemporaryDirectory() as directory:
            root, parent = self.file(directory,'#[cfg(test)]\n#[path="unit.rs"]\nmod support;')
            child = root/'src/unit.rs';child.write_text('pub fn helper() {}')
            with self.assertRaises(ValueError):
                COVERAGE.production_regions(self.export(child,[self.region(1,1,1)]),root)

    def test_external_subtree_cannot_redirect_back_into_src(self):
        with tempfile.TemporaryDirectory() as directory:
            root=pathlib.Path(directory).resolve();(root/'src').mkdir();(root/'tests').mkdir()
            path=root/'src/lib.rs';source='#[cfg(test)] #[path="../tests/support.rs"] mod support;'
            path.write_text(source)
            child=root/'tests/support.rs'
            for text in ['#[path="../src/unit.rs"] mod unit;',
                         'include!("../src/unit.rs");',
                         '#[cfg_attr(test,path="../src/unit.rs")] mod unit;']:
                child.write_text(text)
                with self.subTest(text=text), self.assertRaises(ValueError):
                    COVERAGE.test_only_ranges(source,path,root)

    def test_default_external_and_nested_external_modules_fail(self):
        for source in ['#[cfg(test)] mod support;',
                       '#[cfg(test)] mod unit { mod support; }',
                       '#[cfg(test)] mod unit { mod r#support; }',
                       '#[cfg(test)] mod unit { include!("unit.rs"); }']:
            with self.subTest(source=source), self.assertRaises(ValueError):
                COVERAGE.test_only_ranges(source)

    def test_exclusion_preflight_checks_files_without_regions(self):
        with tempfile.TemporaryDirectory() as directory:
            root, path = self.file(directory,'fn after() {}')
            (root/'src/no_regions.rs').write_text('#![cfg(test)]\nfn hidden() {}')
            with self.assertRaises(ValueError):
                COVERAGE.production_regions(self.export(path,[self.region(1,1,1)]),root)

    def test_actual_region_merge_kind_filter_and_outside_root(self):
        with tempfile.TemporaryDirectory() as directory:
            root,path = self.file(directory,'fn before() {}\n#[cfg(test)]\nfn helper() {}\nfn after() {}')
            exported=self.export(path,[self.region(1,1,0),self.region(3,1,9),self.region(4,1,0)])
            exported['data'][0]['functions'].append(self.function(path,[self.region(1,1,3),self.region(4,1,7,1)]))
            report,scopes=COVERAGE.production_regions(exported,root)
            self.assertEqual(scopes,{'src/sample.rs':[(2,1,3,15)]})
            self.assertEqual(report,{'src/sample.rs':{'regions':2,'covered':1,'uncovered_regions':[[4,1,4,2]]}})

    def test_current_source_gates_are_supported(self):
        root=COVERAGE.ROOT
        files=sorted((root/'src').rglob('*.rs'))
        self.assertTrue(files,'current source test must not pass vacuously')
        gates=0
        for path in files:
            spans=COVERAGE.test_only_ranges(path.read_text(),path,root)
            gates+=len(spans)
            self.assertTrue(all((a,b)<(c,d) for a,b,c,d in spans))
        self.assertGreater(gates,0)

    @staticmethod
    def file(directory,source):
        root=pathlib.Path(directory).resolve();path=root/'src/sample.rs';path.parent.mkdir();path.write_text(source)
        return root,path

    @staticmethod
    def region(line,column,count,kind=0):
        return [line,column,line,column+1,count,0,0,kind]

    @staticmethod
    def function(path,regions):
        return {'filenames':[str(path)],'regions':regions}

    @classmethod
    def export(cls,path,regions):
        return {'data':[{'functions':[cls.function(path,regions)]}]}


if __name__=='__main__':
    unittest.main()
