// Parse the pinned src/circuit_description dump of the Orchard Action
// circuit verifier key and emit a Markdown appendix listing every gate
// polynomial as KaTeX. The input is the Rust Debug rendering of
// halo2_proofs::plonk::PinnedVerificationKey at the orchard 0.13.1
// commit; the format is stable between halo2_proofs minor releases.

use std::env;
use std::fs;
use std::process;

#[derive(Debug, Clone)]
enum Node {
    Call(String, Vec<Node>),
    Struct(String, Vec<(String, Node)>),
    Array(Vec<Node>),
    Hex(String),
    Int(String),
    Str(String),
    Ident(String),
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            s: s.as_bytes(),
            i: 0,
        }
    }

    fn skip_ws(&mut self) {
        while self.i < self.s.len() {
            let c = self.s[self.i];
            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                self.i += 1;
            } else {
                break;
            }
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip_ws();
        if self.i < self.s.len() {
            Some(self.s[self.i])
        } else {
            None
        }
    }

    fn eat(&mut self, c: u8) -> bool {
        self.skip_ws();
        if self.i < self.s.len() && self.s[self.i] == c {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, c: u8) {
        if !self.eat(c) {
            panic!(
                "expected '{}' at offset {} (got {:?})",
                c as char,
                self.i,
                self.s.get(self.i).map(|&b| b as char)
            );
        }
    }

    fn parse_ident(&mut self) -> String {
        self.skip_ws();
        let start = self.i;
        while self.i < self.s.len() {
            let c = self.s[self.i];
            let is_ident = c.is_ascii_alphanumeric() || c == b'_';
            if !is_ident {
                break;
            }
            self.i += 1;
        }
        std::str::from_utf8(&self.s[start..self.i]).unwrap().to_string()
    }

    fn parse_expr(&mut self) -> Node {
        let c = self.peek().expect("unexpected EOF");
        if c == b'"' {
            self.i += 1;
            let start = self.i;
            while self.i < self.s.len() && self.s[self.i] != b'"' {
                if self.s[self.i] == b'\\' && self.i + 1 < self.s.len() {
                    self.i += 2;
                } else {
                    self.i += 1;
                }
            }
            let val = std::str::from_utf8(&self.s[start..self.i]).unwrap().to_string();
            self.i += 1;
            return Node::Str(val);
        }
        if c == b'[' {
            self.i += 1;
            let mut items = vec![];
            while !self.eat(b']') {
                items.push(self.parse_expr());
                self.eat(b',');
            }
            return Node::Array(items);
        }
        if c.is_ascii_digit() || c == b'-' {
            if c == b'0' && self.i + 1 < self.s.len() && self.s[self.i + 1] == b'x' {
                let start = self.i;
                self.i += 2;
                while self.i < self.s.len() && self.s[self.i].is_ascii_hexdigit() {
                    self.i += 1;
                }
                return Node::Hex(std::str::from_utf8(&self.s[start..self.i]).unwrap().to_string());
            }
            let start = self.i;
            if self.s[self.i] == b'-' {
                self.i += 1;
            }
            while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
                self.i += 1;
            }
            return Node::Int(std::str::from_utf8(&self.s[start..self.i]).unwrap().to_string());
        }
        let name = self.parse_ident();
        if self.peek() == Some(b'(') {
            self.i += 1;
            let mut args = vec![];
            while !self.eat(b')') {
                args.push(self.parse_expr());
                self.eat(b',');
            }
            return Node::Call(name, args);
        }
        if self.peek() == Some(b'{') {
            self.i += 1;
            let mut fields = vec![];
            while !self.eat(b'}') {
                let field_name = self.parse_ident();
                self.expect(b':');
                let val = self.parse_expr();
                fields.push((field_name, val));
                self.eat(b',');
            }
            return Node::Struct(name, fields);
        }
        Node::Ident(name)
    }
}

fn extract_gates(s: &str) -> Vec<Node> {
    let marker = "gates: [";
    let idx = s
        .find(marker)
        .expect("could not locate the gates: [ marker; is this a PinnedVerificationKey dump?");
    let start = idx + marker.len() - 1;
    let mut p = Parser::new(&s[start..]);
    match p.parse_expr() {
        Node::Array(items) => items,
        other => panic!("expected an Array of gates at the marker, got {:?}", other),
    }
}

fn field_int(fields: &[(String, Node)], name: &str) -> i64 {
    for (k, v) in fields {
        if k == name {
            if let Node::Int(s) = v {
                return s.parse().unwrap_or(0);
            }
        }
    }
    panic!("field {} not found in struct", name);
}

fn field_rotation(fields: &[(String, Node)]) -> i64 {
    for (k, v) in fields {
        if k == "rotation" {
            if let Node::Call(_, args) = v {
                if let Some(Node::Int(s)) = args.first() {
                    return s.parse().unwrap_or(0);
                }
            }
        }
    }
    0
}

fn shorten_hex(h: &str) -> String {
    let digits = h.strip_prefix("0x").unwrap_or(h);
    let trimmed = digits.trim_start_matches('0');
    if trimmed.is_empty() {
        "0".to_string()
    } else if trimmed.len() <= 4 {
        format!("\\mathtt{{0x{}}}", trimmed)
    } else {
        format!("\\mathtt{{0x{}\\ldots}}", &trimmed[..6])
    }
}

fn rotation_suffix(r: i64) -> String {
    if r == 0 {
        String::new()
    } else if r > 0 {
        format!("^{{(+{})}}", r)
    } else {
        format!("^{{({})}}", r)
    }
}

/// A compressed-selector envelope looks like
/// `(F_c) * (k_1 - F_c) * (k_2 - F_c) * ... * (k_n - F_c) * P`
/// where P is the actual gate body. `compress_selectors` builds this so
/// that exactly one value of `F_c` activates each member of the gate
/// group: the body is enforced when `F_c` takes the value that makes
/// every `(k_i - F_c)` factor non-zero AND every other body inactive.
/// We extract `(c, body)` here, treating polynomials that share `c` as
/// belonging to the same source-level `create_gate` call.
fn split_envelope(node: &Node) -> Option<(i64, &Node)> {
    let Node::Call(name, args) = node else { return None };
    if name != "Product" || args.len() != 2 {
        return None;
    }
    if let Some(c) = find_envelope_col(&args[0]) {
        return Some((c, &args[1]));
    }
    if let Some(c) = find_envelope_col(&args[1]) {
        return Some((c, &args[0]));
    }
    None
}

/// Walk a left-side Product chain looking for any envelope factor
/// `F_c` or `(k - F_c)`; return the first column index `c` seen. The
/// chain may contain non-envelope factors (e.g. an Advice column that
/// gates the constraint on top of the selector), as long as at least
/// one envelope factor is present.
fn find_envelope_col(node: &Node) -> Option<i64> {
    if let Some(c) = envelope_factor_col(node) {
        return Some(c);
    }
    let Node::Call(name, args) = node else { return None };
    if name != "Product" || args.len() != 2 {
        return None;
    }
    if let Some(c) = find_envelope_col(&args[0]) {
        return Some(c);
    }
    find_envelope_col(&args[1])
}

fn envelope_factor_col(node: &Node) -> Option<i64> {
    // Matches either Fixed{column_index: c, rotation: 0} or
    // Sum(Constant(k), Negated(Fixed{column_index: c, rotation: 0})).
    if let Node::Struct(name, fields) = node {
        if name == "Fixed" && field_rotation(fields) == 0 {
            return Some(field_int(fields, "column_index"));
        }
    }
    if let Node::Call(name, args) = node {
        if name == "Sum" && args.len() == 2 {
            if let (Node::Call(cn, cargs), Node::Call(nn, nargs)) = (&args[0], &args[1]) {
                if cn == "Constant" && nn == "Negated" && nargs.len() == 1 {
                    if let Node::Struct(sn, sfields) = &nargs[0] {
                        if sn == "Fixed"
                            && field_rotation(sfields) == 0
                            && matches!(cargs.first(), Some(Node::Hex(_) | Node::Int(_)))
                        {
                            return Some(field_int(sfields, "column_index"));
                        }
                    }
                }
            }
        }
    }
    None
}

fn to_latex(node: &Node) -> String {
    match node {
        Node::Call(name, args) => match name.as_str() {
            "Product" if args.len() == 2 => format!(
                "\\left({}\\right) \\cdot \\left({}\\right)",
                to_latex(&args[0]),
                to_latex(&args[1])
            ),
            "Sum" if args.len() == 2 => {
                format!("{} + {}", to_latex(&args[0]), to_latex(&args[1]))
            }
            "Negated" if args.len() == 1 => {
                format!("-\\left({}\\right)", to_latex(&args[0]))
            }
            "Constant" if args.len() == 1 => to_latex(&args[0]),
            "Scaled" if args.len() == 2 => format!(
                "{} \\cdot \\left({}\\right)",
                to_latex(&args[1]),
                to_latex(&args[0])
            ),
            "Selector" if args.len() == 1 => format!("S_{{{}}}", to_latex(&args[0])),
            "Rotation" if args.len() == 1 => to_latex(&args[0]),
            _ => format!(
                "\\mathsf{{{}}}({})",
                name,
                args.iter()
                    .map(to_latex)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        },
        Node::Struct(name, fields) => match name.as_str() {
            "Fixed" => {
                let c = field_int(fields, "column_index");
                let r = field_rotation(fields);
                format!("F_{{{}}}{}", c, rotation_suffix(r))
            }
            "Advice" => {
                let c = field_int(fields, "column_index");
                let r = field_rotation(fields);
                format!("A_{{{}}}{}", c, rotation_suffix(r))
            }
            "Instance" => {
                let c = field_int(fields, "column_index");
                let r = field_rotation(fields);
                format!("I_{{{}}}{}", c, rotation_suffix(r))
            }
            _ => format!("\\mathsf{{{}}}\\{{\\ldots\\}}", name),
        },
        Node::Hex(h) => shorten_hex(h),
        Node::Int(s) => s.clone(),
        Node::Str(s) => format!("\\text{{{}}}", s),
        Node::Ident(s) => format!("\\mathsf{{{}}}", s),
        Node::Array(_) => "[\\ldots]".to_string(),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let default_input =
        "../../data/orchard-0.13.1-circuit_description.txt".to_string();
    let in_path = args.get(1).unwrap_or(&default_input);
    let input = match fs::read_to_string(in_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read {}: {}", in_path, e);
            process::exit(1);
        }
    };
    let gates = extract_gates(&input);

    println!("---");
    println!("sidebar_position: 21");
    println!("title: \"Appendix: Action Circuit Polynomial Constraints\"");
    println!(
        "description: Auto-generated KaTeX rendering of every gate \
         polynomial in the Orchard 0.13.1 Action circuit verifier key."
    );
    println!("---");
    println!();
    println!("# Appendix: Action Circuit Polynomial Constraints");
    println!();
    println!(
        "This appendix lists the {} polynomial constraints of the",
        gates.len()
    );
    println!("Orchard Action circuit at orchard 0.13.1. Each polynomial $P$");
    println!("vanishes on every valid assignment: $P = 0$.");
    println!();
    println!("**Provenance.** The polynomials are extracted from the");
    println!("[`src/circuit_description`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit_description)");
    println!("dump (a serialisation of");
    println!("`halo2_proofs::plonk::PinnedVerificationKey`) by the");
    println!("[`gates-to-latex`](https://github.com/dannywillems/orchard/tree/onboarding/onboarding/tools/gates-to-latex)");
    println!("tool that ships in this repo. To regenerate after an upstream");
    println!("change, run `make appendix-gates` from the `onboarding/`");
    println!("directory; the tool re-reads the vendored copy at");
    println!("`onboarding/data/orchard-0.13.1-circuit_description.txt`.");
    println!();
    println!("**Notation.** The advice, fixed, and instance columns are");
    println!("indexed by their `column_index` in the constraint system:");
    println!();
    println!("- $A_c$, $A_c^{{(+r)}}$, $A_c^{{(-r)}}$: advice column $c$ at the");
    println!("  current row, rotated by $+r$ or $-r$.");
    println!("- $F_c$, $F_c^{{(+r)}}$, $F_c^{{(-r)}}$: fixed column $c$ at the");
    println!("  current row or a rotation. The pinned circuit uses 29 fixed");
    println!("  columns; the lowest indices are the selector-promotion");
    println!("  columns produced by Halo 2's `compress_selectors` pass, and");
    println!("  the higher indices carry the chip-level constants used by");
    println!("  the ECC, Sinsemilla, and Poseidon chips.");
    println!("- Constants are rendered in hex. Values below `0xffff` are");
    println!("  shown in full; larger values are truncated to a six-hex-digit");
    println!("  head followed by `\\ldots` to keep KaTeX expressions readable.");
    println!();
    println!("**Grouping.** Halo 2's `compress_selectors` pass packs every");
    println!("`meta.create_gate(...)` group into a single shared fixed column");
    println!("by giving the column a small integer value per gate member.");
    println!("That value selects the member through an envelope of the form");
    println!("$F_c \\cdot (k_1 - F_c) \\cdot \\dots \\cdot (k_n - F_c)$. Two");
    println!("polynomials that share the same envelope column $c$ therefore");
    println!("come from the same source-level `create_gate` call. We use $c$");
    println!("as the group key and list polynomials per group; the");
    println!("source-level chip that owns each group can be identified by");
    println!("opening `src/circuit.rs` and reading the chip-configuration");
    println!("calls in `Circuit::configure` in order. Polynomials that do");
    println!("not match the envelope pattern are listed under \"Ungrouped\".");
    println!();
    println!("**Scope.** This is the raw polynomial form, not yet annotated");
    println!("with chip-level meaning. Phase 2 of this work would attach a");
    println!("source-level chip name to each group (ECC, Sinsemilla,");
    println!("Poseidon, Merkle, CommitIvk, NoteCommit). Doing so cleanly");
    println!("requires upstream changes in `halo2_proofs` to expose gate");
    println!("names; the pinned dump deliberately strips them.");
    println!();

    // Collect (group_id, original_index, polynomial).
    let mut groups: std::collections::BTreeMap<Option<i64>, Vec<(usize, &Node)>> =
        std::collections::BTreeMap::new();
    for (i, g) in gates.iter().enumerate() {
        let key = split_envelope(g).map(|(c, _)| c);
        groups.entry(key).or_default().push((i, g));
    }

    println!("## Summary");
    println!();
    println!("| Envelope column $c$ | Polynomials in group | Original indices                   |");
    println!("| ------------------- | -------------------- | ---------------------------------- |");
    for (k, v) in &groups {
        let key_str = match k {
            Some(c) => format!("$F_{{{}}}$", c),
            None => "ungrouped".to_string(),
        };
        let indices = v
            .iter()
            .map(|(i, _)| (i + 1).to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let indices = if indices.len() > 60 {
            format!("{}\\,...", &indices[..60])
        } else {
            indices
        };
        println!("| {} | {} | {} |", key_str, v.len(), indices);
    }
    println!();

    let mut group_no = 0;
    for (k, v) in &groups {
        group_no += 1;
        let header = match k {
            Some(c) => format!(
                "## Group {} (envelope column $F_{{{}}}$, {} polynomials)",
                group_no,
                c,
                v.len()
            ),
            None => format!(
                "## Group {} (ungrouped, {} polynomials)",
                group_no,
                v.len()
            ),
        };
        println!("{}", header);
        println!();
        for (idx, gate) in v {
            println!("### Polynomial {} (original index {})", idx + 1, idx + 1);
            println!();
            // Print the envelope-stripped body when we have one, so the
            // selector clutter is moved out of the math block.
            let body = split_envelope(gate).map(|(_, b)| b).unwrap_or(gate);
            println!("$$");
            println!("{} = 0", to_latex(body));
            println!("$$");
            println!();
        }
    }
}
