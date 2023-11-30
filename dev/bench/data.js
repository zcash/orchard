window.BENCHMARK_DATA = {
  "lastUpdate": 1701332527018,
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
          "id": "003619bad4b7d8ae2f128a86a9ae9da7b9947fc1",
          "message": "Merge pull request #402 from zcash/ci-updates\n\nCI: Modernise workflows",
          "timestamp": "2023-11-30T08:04:40Z",
          "tree_id": "6b18a94ff863b8c151d415fa0cd53316476c1982",
          "url": "https://github.com/zcash/orchard/commit/003619bad4b7d8ae2f128a86a9ae9da7b9947fc1"
        },
        "date": 1701332525611,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2935292550,
            "range": "± 38008911",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2928236172,
            "range": "± 21506445",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4185494822,
            "range": "± 31393316",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5425596682,
            "range": "± 22419758",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25330960,
            "range": "± 1204691",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25306634,
            "range": "± 528292",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28235500,
            "range": "± 546231",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31673285,
            "range": "± 446184",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1525213,
            "range": "± 10381",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127462,
            "range": "± 559",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1521561,
            "range": "± 5311",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1352763978,
            "range": "± 4965675",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16113852,
            "range": "± 45777",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2169024,
            "range": "± 13081",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16074561,
            "range": "± 208410",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2140056,
            "range": "± 8136",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80513753,
            "range": "± 150756",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10795829,
            "range": "± 57304",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80339868,
            "range": "± 416308",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10655131,
            "range": "± 385694",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161082169,
            "range": "± 2247513",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21592194,
            "range": "± 89409",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160637086,
            "range": "± 2477606",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21289848,
            "range": "± 119277",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 467661,
            "range": "± 16733",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 501592,
            "range": "± 1839",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}