window.BENCHMARK_DATA = {
  "lastUpdate": 1704561400976,
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
          "id": "e127230af008006eecf6ecdfb8a265ee059a6a2e",
          "message": "Merge pull request #411 from nuttycom/public_merkle_depth\n\nMake the `MERKLE_DEPTH_ORCHARD` constant public.",
          "timestamp": "2024-01-06T10:02:15-07:00",
          "tree_id": "7663a40d9d111bc5b6dfebfc8abc3109578664cc",
          "url": "https://github.com/zcash/orchard/commit/e127230af008006eecf6ecdfb8a265ee059a6a2e"
        },
        "date": 1704561399857,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2899701543,
            "range": "± 290692541",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2891118585,
            "range": "± 33628317",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4153309905,
            "range": "± 28719213",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5415045352,
            "range": "± 20550705",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25134996,
            "range": "± 1289471",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25019474,
            "range": "± 589961",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28008034,
            "range": "± 608303",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31781816,
            "range": "± 445380",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1530385,
            "range": "± 13699",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 128145,
            "range": "± 1488",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1525312,
            "range": "± 11100",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1343389934,
            "range": "± 4156944",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16157175,
            "range": "± 202159",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2181720,
            "range": "± 17567",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16095707,
            "range": "± 91677",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2143505,
            "range": "± 22951",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80696115,
            "range": "± 941096",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10829588,
            "range": "± 101186",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80375224,
            "range": "± 942640",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10653245,
            "range": "± 137900",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161479251,
            "range": "± 831625",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21673468,
            "range": "± 46143",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160813492,
            "range": "± 280789",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21298797,
            "range": "± 58462",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 466647,
            "range": "± 5423",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 502499,
            "range": "± 1306",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}