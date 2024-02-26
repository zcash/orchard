window.BENCHMARK_DATA = {
  "lastUpdate": 1708971733115,
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
          "id": "f8857e82924bfadcb8cc32869e56083f12ab2a0c",
          "message": "Merge pull request #418 from daira/action-doc-nit\n\nTrivial update to the doc for `struct Action`",
          "timestamp": "2024-02-26T18:05:50Z",
          "tree_id": "c69cdd09ad6997a5e5ae60fc2e04215e8a40fcae",
          "url": "https://github.com/zcash/orchard/commit/f8857e82924bfadcb8cc32869e56083f12ab2a0c"
        },
        "date": 1708971731430,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2906705055,
            "range": "± 149194374",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2897510646,
            "range": "± 11461689",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4146010203,
            "range": "± 12507811",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5414953411,
            "range": "± 15231184",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25203510,
            "range": "± 488979",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25351657,
            "range": "± 533627",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28184097,
            "range": "± 544848",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31511226,
            "range": "± 393065",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1530697,
            "range": "± 7188",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 128197,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1528203,
            "range": "± 22389",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1359829594,
            "range": "± 3392593",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16167459,
            "range": "± 15107",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2186490,
            "range": "± 1856",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16142314,
            "range": "± 61664",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2153844,
            "range": "± 7053",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80776332,
            "range": "± 268411",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10880655,
            "range": "± 25112",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80633931,
            "range": "± 240698",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10710166,
            "range": "± 37724",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161572413,
            "range": "± 339035",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21742680,
            "range": "± 1031363",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 161308001,
            "range": "± 539502",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21398536,
            "range": "± 65808",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 466200,
            "range": "± 2003",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 500411,
            "range": "± 1566",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}