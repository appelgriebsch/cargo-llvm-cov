// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::Result;
use regex::Regex;

pub(crate) struct RegexBuilder {
    re: String,
    first: bool,
    trailing: &'static str,
}

impl RegexBuilder {
    pub(crate) fn new(leading: &'static str, trailing: &'static str) -> Self {
        let mut re = String::with_capacity(256);
        re.push_str(leading);
        Self { re, first: true, trailing }
    }

    pub(crate) fn or(&mut self, pat: &str) {
        if self.first {
            self.first = false;
        } else {
            self.re.push('|');
        }
        self.re.push_str(pat);
    }

    pub(crate) fn build(mut self) -> Result<Regex> {
        self.re.push_str(self.trailing);
        Ok(Regex::new(&self.re)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn smoke() {
        let mut re = RegexBuilder::new("^(", ")$");
        re.or(&"a".repeat(64 * 4100));
        let _re = re.build().unwrap();

        let mut re = RegexBuilder::new("^(", ")$");
        for _ in 0..4096 {
            re.or("a");
        }
        re.or("b");
        re.or("c");
        let re = re.build().unwrap();
        assert!(re.is_match("a"));
        assert!(!re.is_match("aa"));
        assert!(re.is_match("b"));
        assert!(!re.is_match("bb"));
        assert!(re.is_match("c"));
        assert!(!re.is_match("cc"));
        assert!(!re.is_match("d"));
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn pkg_hash_re_size_limit() {
        const MID_PKG_NAME: (usize, usize) = (20000, 64);
        const LONG_PKG_NAME: (usize, usize) = (10000, 128);
        const TOO_LONG_PKG_NAME: (usize, usize) = (5000, 256);

        fn gen_pkg_names((num_pkg, pkg_name_size): (usize, usize)) -> Vec<String> {
            (0..num_pkg)
                .map(|n| ('a'..='z').cycle().skip(n).take(pkg_name_size).collect())
                .collect::<Vec<_>>()
        }

        fn pkg_hash_re(pkg_names: &[String]) -> Result<Regex> {
            let mut re = RegexBuilder::new("^(lib)?(", ")(-[0-9a-f]{7,})?$");
            for name in pkg_names {
                re.or(&name.replace('-', "(-|_)"));
            }
            re.build()
        }

        pkg_hash_re(&gen_pkg_names(MID_PKG_NAME)).unwrap();
        pkg_hash_re(&gen_pkg_names(LONG_PKG_NAME)).unwrap();
        pkg_hash_re(&gen_pkg_names(TOO_LONG_PKG_NAME)).unwrap();
    }
}
