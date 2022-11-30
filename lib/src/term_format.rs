use crate::{
    rule::Rule,
    rules::Rules,
    verifier::{Verificators, Verifier},
};
use pad::PadStr;
use std::ops::Deref;

pub trait TermFormat {
    fn formatted(&self) -> String;
}

impl TermFormat for Rule {
    fn formatted(&self) -> String {
        self.to_string()
            .pad_to_width_with_alignment(25, pad::Alignment::Left)
    }
}

impl TermFormat for Rules {
    fn formatted(&self) -> String {
        let mut s = String::new();
        for r in self.deref() {
            s.push_str(&r.formatted());
            s.push('\n');
        }
        s
    }
}

impl TermFormat for Verifier {
    fn formatted(&self) -> String {
        let mut s = String::new();
        s.push_str(&self.rule.formatted());
        s.push_str(": ");
        s.push_str(&self.mask.to_string());
        s.push_str(" (");
        s.push_str(format!("{}", &self.mask.count_ones()).as_str());
        s.push(')');
        s.push('\n');

        s
    }
}

impl TermFormat for Verificators {
    fn formatted(&self) -> String {
        let mut s = String::new();
        for v in self.deref() {
            s.push_str(&v.formatted());
        }
        s
    }
}
