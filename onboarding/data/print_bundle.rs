// Drop-in snippet for tests/builder.rs that prints every field of
// an Authorized Bundle at the byte level. Match the printed lengths
// against the table in worked-example.md Section 2. Hex values
// differ between runs because the prover samples fresh randomness;
// the lengths are invariant.
//
// Usage:
//   1. Open tests/builder.rs in zcash/orchard at tag 0.13.1.
//   2. After the first bundle is built and authorized, paste the
//      lines below in place of (or alongside) the existing
//      verify_bundle(&bundle, ...) call.
//   3. cargo test --release --test builder -- --nocapture

let actions = bundle.actions();
for (i, a) in actions.iter().enumerate() {
    println!("action {i}:");
    println!("  cv_net = {}", hex::encode(a.cv_net().to_bytes()));
    println!("  nf     = {}", hex::encode(a.nullifier().to_bytes()));
    println!("  rk     = {}", hex::encode(<[u8; 32]>::from(a.rk())));
    println!("  cmx    = {}", hex::encode(a.cmx().to_bytes()));
    let ct = a.encrypted_note();
    println!("  epk    = {}", hex::encode(ct.epk_bytes));
    println!(
        "  enc    = {} ({} bytes)",
        &hex::encode(ct.enc_ciphertext)[..32],
        ct.enc_ciphertext.len()
    );
    println!(
        "  out    = {} ({} bytes)",
        &hex::encode(ct.out_ciphertext)[..32],
        ct.out_ciphertext.len()
    );
}
println!("flags          = {:08b}", bundle.flags().to_byte());
println!("value_balance  = {}", bundle.value_balance());
println!("anchor         = {}", hex::encode(bundle.anchor().to_bytes()));
println!(
    "proof length   = {} bytes",
    bundle.authorization().proof().as_ref().len()
);
