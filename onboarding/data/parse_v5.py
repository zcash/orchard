#!/usr/bin/env python3
"""Parse a Zcash v5 (NU5+) transaction hex and dump the layout.

The transaction format is specified in ZIP 225 (Version 5
Transaction Format) at https://zips.z.cash/zip-0225. The Orchard
region is the subject of the dump.

Pass an index into zip_0244.json as the only argument; only
vectors 2, 4, 5, 6 contain Orchard actions.

Example:
    python3 parse_v5.py 2

The JSON file is the official ZIP 244 test corpus, mirrored from
https://github.com/zcash-hackworks/zcash-test-vectors and shipped
in this repository at onboarding/data/zip_0244.json so the
example does not depend on an external URL.
"""

import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
JSON_PATH = os.path.join(HERE, 'zip_0244.json')


def read(buf, off, n):
    return buf[off : off + n], off + n


def read_compact_size(buf, off):
    first, off = read(buf, off, 1)
    f = first[0]
    if f < 0xFD:
        return f, off
    if f == 0xFD:
        b, off = read(buf, off, 2)
        return int.from_bytes(b, 'little'), off
    if f == 0xFE:
        b, off = read(buf, off, 4)
        return int.from_bytes(b, 'little'), off
    b, off = read(buf, off, 8)
    return int.from_bytes(b, 'little'), off


def parse(tx_hex, label=''):
    raw = bytes.fromhex(tx_hex)
    off = 0
    print(f'tx label: {label}')
    print(f'tx bytes: {len(raw)}')

    header, off = read(raw, off, 4)
    print(f'header                4 bytes  {header.hex()}  (v5+overwinter bit)')
    version_group_id, off = read(raw, off, 4)
    print(f'nVersionGroupId       4 bytes  {version_group_id.hex()}')
    consensus_branch_id, off = read(raw, off, 4)
    print(f'nConsensusBranchId    4 bytes  {consensus_branch_id.hex()}')
    lock_time, off = read(raw, off, 4)
    print(f'lock_time             4 bytes  {lock_time.hex()}')
    expiry_height, off = read(raw, off, 4)
    print(f'nExpiryHeight         4 bytes  {expiry_height.hex()}')

    tx_in_count, off = read_compact_size(raw, off)
    print(f'tx_in_count (CS)      {tx_in_count}')
    for _ in range(tx_in_count):
        _prev, off = read(raw, off, 36)
        scr_len, off = read_compact_size(raw, off)
        _scr, off = read(raw, off, scr_len)
        _seq, off = read(raw, off, 4)

    tx_out_count, off = read_compact_size(raw, off)
    print(f'tx_out_count (CS)     {tx_out_count}')
    for _ in range(tx_out_count):
        _val, off = read(raw, off, 8)
        scr_len, off = read_compact_size(raw, off)
        _scr, off = read(raw, off, scr_len)

    n_spends_sap, off = read_compact_size(raw, off)
    print(f'nSpendsSapling (CS)   {n_spends_sap}')
    for _ in range(n_spends_sap):
        _spend, off = read(raw, off, 32 + 32 + 32 + 192)

    n_outputs_sap, off = read_compact_size(raw, off)
    print(f'nOutputsSapling (CS)  {n_outputs_sap}')
    for _ in range(n_outputs_sap):
        _o, off = read(raw, off, 32 + 32 + 32 + 580 + 80 + 192)

    if n_spends_sap > 0 or n_outputs_sap > 0:
        _vbs, off = read(raw, off, 8)
    if n_spends_sap > 0:
        _anc_sap, off = read(raw, off, 32)
    for _ in range(n_spends_sap):
        _sig, off = read(raw, off, 64)
    if n_spends_sap > 0 or n_outputs_sap > 0:
        _bind_sap, off = read(raw, off, 64)

    n_actions, off = read_compact_size(raw, off)
    print('\n==== ORCHARD REGION ====')
    print(f'nActionsOrchard (CS)  {n_actions}')
    if n_actions == 0:
        print('(no Orchard actions in this transaction)')
        return

    actions_start = off
    for i in range(n_actions):
        a_off = off
        cv_net, off = read(raw, off, 32)
        nf, off = read(raw, off, 32)
        rk, off = read(raw, off, 32)
        cmx, off = read(raw, off, 32)
        epk, off = read(raw, off, 32)
        enc, off = read(raw, off, 580)
        out, off = read(raw, off, 80)
        print(f'\n  Action[{i}] (820 bytes, starts at offset {a_off})')
        print(f'    cv_net  (+0  ,32)  {cv_net.hex()}')
        print(f'    nf      (+32 ,32)  {nf.hex()}')
        print(f'    rk      (+64 ,32)  {rk.hex()}')
        print(f'    cmx     (+96 ,32)  {cmx.hex()}')
        print(f'    epk     (+128,32)  {epk.hex()}')
        print(f'    enc[0:16] (+160)   {enc[:16].hex()}...')
        print(f'    enc tag (+724,16)  {enc[-16:].hex()}')
        print(f'    out[0:16] (+740)   {out[:16].hex()}...')
        print(f'    out tag (+804,16)  {out[-16:].hex()}')

    flags, off = read(raw, off, 1)
    print(f'\nflagsOrchard          1 byte   0x{flags.hex()}')
    val_bal, off = read(raw, off, 8)
    val_bal_int = int.from_bytes(val_bal, 'little', signed=True)
    print(f'valueBalanceOrchard   8 bytes  {val_bal.hex()}  ({val_bal_int} zatoshi)')
    anchor, off = read(raw, off, 32)
    print(f'anchorOrchard         32 bytes {anchor.hex()}')
    proof_sz, off = read_compact_size(raw, off)
    proof, off = read(raw, off, proof_sz)
    print(f'sizeProofsOrchard      {proof_sz}')
    print(
        f'proofsOrchard         {proof_sz} bytes  '
        f'{proof[:16].hex()}...{proof[-16:].hex()}'
    )
    for i in range(n_actions):
        sig, off = read(raw, off, 64)
        print(f'spendAuthSig[{i}]        64 bytes  {sig.hex()[:32]}...')
    bind_sig, off = read(raw, off, 64)
    print(f'bindingSigOrchard     64 bytes {bind_sig.hex()[:32]}...')
    print(f'\nleftover bytes (should be 0): {len(raw) - off}')
    print(f'Orchard region total:         {off - actions_start + 1}')


def main():
    if not os.path.exists(JSON_PATH):
        print(f'ERROR: {JSON_PATH} not found.', file=sys.stderr)
        sys.exit(1)
    with open(JSON_PATH) as fp:
        d = json.load(fp)
    if len(sys.argv) < 2:
        print('Usage: parse_v5.py <vector_index>')
        print('  vector indices 0..9; Orchard data lives at 2, 4, 5, 6.')
        sys.exit(2)
    idx = int(sys.argv[1])
    vec = d[2 + idx]
    tx_hex = vec[0]
    txid = vec[1]
    parse(tx_hex, label=f'index {idx}, txid {txid}')


if __name__ == '__main__':
    main()
