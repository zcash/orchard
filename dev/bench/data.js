window.BENCHMARK_DATA = {
  "lastUpdate": 1644894963156,
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
          "id": "d6edf2a21ada92832e0968a88bea0248d79ec085",
          "message": "Merge pull request #284 from zcash/update-halo2-deps\n\nMigrate to `halo2_proofs 0.1.0-beta.2`",
          "timestamp": "2022-02-15T02:54:28Z",
          "tree_id": "98c10479274d4332ab3599da8d9dc9be8dd7a9d1",
          "url": "https://github.com/zcash/orchard/commit/d6edf2a21ada92832e0968a88bea0248d79ec085"
        },
        "date": 1644894962111,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 5349713647,
            "range": "± 264687073",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 5237359492,
            "range": "± 106023325",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 7648942686,
            "range": "± 132767808",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 10112644201,
            "range": "± 126043350",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 42385012,
            "range": "± 2352465",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 41934660,
            "range": "± 9818116",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 45613731,
            "range": "± 2030825",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 51019756,
            "range": "± 4145317",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1446699,
            "range": "± 92700",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 184559,
            "range": "± 8169",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1295720,
            "range": "± 79209",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 162276148,
            "range": "± 7595815",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 28653297,
            "range": "± 1190172",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 3155115,
            "range": "± 171013",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 25715208,
            "range": "± 1504824",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2896463,
            "range": "± 180540",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 144244852,
            "range": "± 4882356",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 14927131,
            "range": "± 786482",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 126511402,
            "range": "± 6982429",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 13873404,
            "range": "± 767041",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 287579499,
            "range": "± 6990185",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 29712712,
            "range": "± 1561272",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 252904686,
            "range": "± 14825185",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 29306948,
            "range": "± 2061194",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 619255,
            "range": "± 39724",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 651479,
            "range": "± 46846",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}