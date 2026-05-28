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
    println!("**Scope.** This is the raw polynomial form, not yet annotated");
    println!("with chip-level meaning. Phase 2 of this work attaches each");
    println!("polynomial to the chip (ECC, Sinsemilla, Poseidon, Merkle,");
    println!("CommitIvk, NoteCommit) that emitted it and explains what it");
    println!("enforces; until that lands, refer to the column-allocation");
    println!("comments in");
    println!("[`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs)");
    println!("to map a column index back to its owning chip.");
    println!();

    for (idx, gate) in gates.iter().enumerate() {
        println!("## Polynomial {}", idx + 1);
        println!();
        println!("$$");
        println!("{} = 0", to_latex(gate));
        println!("$$");
        println!();
    }
}
