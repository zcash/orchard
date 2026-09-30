window.BENCHMARK_DATA = {
  "lastUpdate": 1790807219955,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "kris@nutty.land",
            "name": "Kris Nuttycombe",
            "username": "nuttycom"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "616a669df8c9a59c33d47064d3ce04b25d026b2e",
          "message": "Merge pull request #564 from zcash/release/0.16.0\n\nRelease orchard v0.16.0",
          "timestamp": "2026-09-30T16:15:05-06:00",
          "tree_id": "317c0c894588020a2162b80af13e6c5d0411cac2",
          "url": "https://github.com/zcash/orchard/commit/616a669df8c9a59c33d47064d3ce04b25d026b2e"
        },
        "date": 1790807218516,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2547499344,
            "range": "± 56367872",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2517758469,
            "range": "± 3805873",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3638425710,
            "range": "± 15420007",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 4753614874,
            "range": "± 43072328",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 19225333,
            "range": "± 194681",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 19099451,
            "range": "± 297449",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 22277202,
            "range": "± 341166",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 25096474,
            "range": "± 241345",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1163997,
            "range": "± 13195",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 103768,
            "range": "± 323",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1159669,
            "range": "± 12088",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1097947116,
            "range": "± 2195917",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 11718957,
            "range": "± 32816",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 1117819,
            "range": "± 8702",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 11696045,
            "range": "± 41707",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 1084733,
            "range": "± 11796",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 58485267,
            "range": "± 355396",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 5507398,
            "range": "± 20325",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 58373835,
            "range": "± 214963",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 5348804,
            "range": "± 81377",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 116828619,
            "range": "± 258665",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 11006069,
            "range": "± 21125",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 116606005,
            "range": "± 797033",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 10677258,
            "range": "± 134461",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}