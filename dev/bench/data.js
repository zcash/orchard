window.BENCHMARK_DATA = {
  "lastUpdate": 1644969401738,
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
          "id": "f1795f8068c8c83af89af8a061131759f973d656",
          "message": "Merge pull request #286 from zcash/merge-non-consensus-changes\n\nMerge non-consensus changes",
          "timestamp": "2022-02-15T23:36:29Z",
          "tree_id": "ae64272823e67c16e9680add178f397365f9ea3f",
          "url": "https://github.com/zcash/orchard/commit/f1795f8068c8c83af89af8a061131759f973d656"
        },
        "date": 1644969400703,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4932248564,
            "range": "± 100399386",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 4902930040,
            "range": "± 178643132",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 7292226963,
            "range": "± 205193374",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 9056399203,
            "range": "± 143652680",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 36763010,
            "range": "± 4079685",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 38690552,
            "range": "± 3937868",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 42557949,
            "range": "± 9663052",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 44472800,
            "range": "± 5066865",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1415965,
            "range": "± 64328",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 178351,
            "range": "± 11510",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1071957,
            "range": "± 145206",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 155419150,
            "range": "± 12261310",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 27473865,
            "range": "± 1557625",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2979212,
            "range": "± 196594",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 23751517,
            "range": "± 2632393",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2512667,
            "range": "± 358453",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 133467898,
            "range": "± 8018991",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 13213959,
            "range": "± 1812777",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 111252007,
            "range": "± 14006026",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 12587894,
            "range": "± 1388203",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 268421271,
            "range": "± 11018561",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 25789690,
            "range": "± 3019372",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 219227165,
            "range": "± 20064095",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 24891992,
            "range": "± 3409627",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 553017,
            "range": "± 73780",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 606570,
            "range": "± 74811",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}