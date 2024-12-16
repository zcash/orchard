window.BENCHMARK_DATA = {
  "lastUpdate": 1734377475753,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4fc9b5ee5fb148f07ea26462d8ef9392e088f13e",
          "message": "Merge pull request #445 from zcash/circuitless\n\nMove circuit and its dependencies behind a feature flag",
          "timestamp": "2024-12-17T08:17:21+13:00",
          "tree_id": "f975b55c2b061c46f72faac8c4e43761e37e28c1",
          "url": "https://github.com/zcash/orchard/commit/4fc9b5ee5fb148f07ea26462d8ef9392e088f13e"
        },
        "date": 1734377474801,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2930126600,
            "range": "± 228641328",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2884119974,
            "range": "± 7840630",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4138742506,
            "range": "± 29695208",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5418354326,
            "range": "± 17208520",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25139650,
            "range": "± 548625",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25121685,
            "range": "± 1656739",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28247048,
            "range": "± 622658",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31694606,
            "range": "± 975431",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1524824,
            "range": "± 7189",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127026,
            "range": "± 362",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1521493,
            "range": "± 5920",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1327874856,
            "range": "± 1576962",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16083453,
            "range": "± 37116",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2161961,
            "range": "± 10427",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16061597,
            "range": "± 58015",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2119411,
            "range": "± 5875",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80380704,
            "range": "± 215957",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10761576,
            "range": "± 9365",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80286903,
            "range": "± 183775",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10543645,
            "range": "± 26601",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 160739294,
            "range": "± 263559",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21513947,
            "range": "± 51752",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160586002,
            "range": "± 1142342",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21082458,
            "range": "± 81032",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 466196,
            "range": "± 7394",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 501259,
            "range": "± 1252",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}