window.BENCHMARK_DATA = {
  "lastUpdate": 1734629892554,
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
          "id": "9afad02db07d903d3f21b493d51c31b314841746",
          "message": "Merge pull request #450 from zcash/no_std_ci\n\nAdd CI workflow for verifying no_std compatibility.",
          "timestamp": "2024-12-19T10:24:01-07:00",
          "tree_id": "80a6c7ab335a38381839637cb5c2fe6fa1e3262d",
          "url": "https://github.com/zcash/orchard/commit/9afad02db07d903d3f21b493d51c31b314841746"
        },
        "date": 1734629891829,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2923931720,
            "range": "± 33666636",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2908101677,
            "range": "± 31349372",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4195513378,
            "range": "± 25459222",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5442113758,
            "range": "± 22568005",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25662085,
            "range": "± 865175",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25594811,
            "range": "± 532875",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28611601,
            "range": "± 600903",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 32113481,
            "range": "± 400438",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1530297,
            "range": "± 8160",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 126676,
            "range": "± 366",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1527035,
            "range": "± 9722",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1344577129,
            "range": "± 2984543",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16179595,
            "range": "± 59089",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2163469,
            "range": "± 1857",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16134380,
            "range": "± 164737",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2131959,
            "range": "± 5201",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80804449,
            "range": "± 1546544",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10782849,
            "range": "± 65631",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80620195,
            "range": "± 176678",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10605395,
            "range": "± 85309",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161668032,
            "range": "± 338348",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21549022,
            "range": "± 93279",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 161234602,
            "range": "± 305530",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21206350,
            "range": "± 39505",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 465320,
            "range": "± 2216",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 500911,
            "range": "± 883",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}