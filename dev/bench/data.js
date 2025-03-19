window.BENCHMARK_DATA = {
  "lastUpdate": 1742398765327,
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
          "id": "fcb14defd75c0dd79512289c17a1ac46b5001d3a",
          "message": "Merge pull request #461 from daira/ci-fixes-from-460\n\nCI and dependency fixes for `no_std`",
          "timestamp": "2025-03-19T09:26:31-06:00",
          "tree_id": "b5ae77ea88aedae2fbbc4fb02592db0047c3adca",
          "url": "https://github.com/zcash/orchard/commit/fcb14defd75c0dd79512289c17a1ac46b5001d3a"
        },
        "date": 1742398764486,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2878874810,
            "range": "± 276555107",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2845474689,
            "range": "± 3934000",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4115155442,
            "range": "± 38355153",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5337181444,
            "range": "± 49517621",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 24535907,
            "range": "± 311830",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 24496461,
            "range": "± 883407",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 27849567,
            "range": "± 344469",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31330509,
            "range": "± 197749",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1476618,
            "range": "± 10220",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 125552,
            "range": "± 355",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1473744,
            "range": "± 4718",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1322039912,
            "range": "± 3001458",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15609151,
            "range": "± 38405",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2131785,
            "range": "± 8927",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15577158,
            "range": "± 38695",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2095540,
            "range": "± 5174",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 77974440,
            "range": "± 168133",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10600177,
            "range": "± 24673",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 77865261,
            "range": "± 463604",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10425605,
            "range": "± 26405",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 155944115,
            "range": "± 176465",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21188889,
            "range": "± 38985",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 155632063,
            "range": "± 465310",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20836791,
            "range": "± 50080",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 464004,
            "range": "± 14106",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 487915,
            "range": "± 1538",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}