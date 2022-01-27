window.BENCHMARK_DATA = {
  "lastUpdate": 1643307320803,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "str4d",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "159ab53da59ba762794862dc4569c0273c67ed1c",
          "message": "Merge pull request #186 from zcash/refactor-gadget-crates\n\nPrepare to extract gadgets into crates",
          "timestamp": "2022-01-27T17:53:18Z",
          "tree_id": "974202621179ce45710a1c5ef470df6999e72f63",
          "url": "https://github.com/zcash/orchard/commit/159ab53da59ba762794862dc4569c0273c67ed1c"
        },
        "date": 1643307319410,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 3779018054,
            "range": "± 29483196",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 3778058848,
            "range": "± 6048574",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 5426130417,
            "range": "± 17936320",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 7051806102,
            "range": "± 18661577",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 32228603,
            "range": "± 249670",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 32169327,
            "range": "± 277149",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 35403824,
            "range": "± 349992",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 39850111,
            "range": "± 203146",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 929814,
            "range": "± 572",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 118240,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 927276,
            "range": "± 378",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 121077633,
            "range": "± 34511",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 18159038,
            "range": "± 9235",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2089374,
            "range": "± 607",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 18254787,
            "range": "± 10355",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2057189,
            "range": "± 535",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 91438988,
            "range": "± 32148",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10398696,
            "range": "± 4325",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 103394545,
            "range": "± 48543",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10236748,
            "range": "± 5277",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 182903513,
            "range": "± 71647",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 20786949,
            "range": "± 7176",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 182455047,
            "range": "± 55800",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20456468,
            "range": "± 12255",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 58365093,
            "range": "± 10052743",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3137484,
            "range": "± 47049",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 138264100,
            "range": "± 3454319",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4669302,
            "range": "± 27641",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 207884375,
            "range": "± 4255969",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5622242,
            "range": "± 45083",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 37774,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 139269,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 133836,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 221646,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 221612,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 125482,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 136501,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 224233,
            "range": "± 472",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 224262,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 262645,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 273680,
            "range": "± 301",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 361475,
            "range": "± 261",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 361481,
            "range": "± 211",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 440254,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 497920,
            "range": "± 304",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}