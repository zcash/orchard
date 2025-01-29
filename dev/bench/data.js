window.BENCHMARK_DATA = {
  "lastUpdate": 1738121953563,
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
          "id": "c684e9185a0449efb00428f807d3bf286b5dae03",
          "message": "Merge pull request #453 from zcash/circuit_test_deps\n\nMove circuit-dependent test utils behind the `circuit` feature.",
          "timestamp": "2025-01-28T20:26:17-07:00",
          "tree_id": "4db58a438c31478426c204921b887f33f0ca75f6",
          "url": "https://github.com/zcash/orchard/commit/c684e9185a0449efb00428f807d3bf286b5dae03"
        },
        "date": 1738121952731,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2900810250,
            "range": "± 248189706",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2879180196,
            "range": "± 11384958",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4110623956,
            "range": "± 7575398",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5331922820,
            "range": "± 39484783",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 24381873,
            "range": "± 278790",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 24446643,
            "range": "± 653383",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 27568973,
            "range": "± 553367",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31283952,
            "range": "± 312375",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1479815,
            "range": "± 9839",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 125679,
            "range": "± 217",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1477538,
            "range": "± 11211",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1333618978,
            "range": "± 3921204",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15648126,
            "range": "± 524114",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2133687,
            "range": "± 4692",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15618247,
            "range": "± 36315",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2100278,
            "range": "± 5770",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 78171335,
            "range": "± 178712",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10616526,
            "range": "± 25653",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 78040607,
            "range": "± 375440",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10446484,
            "range": "± 24694",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 156356358,
            "range": "± 2537914",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21213877,
            "range": "± 39877",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 155989760,
            "range": "± 1277562",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20873848,
            "range": "± 37081",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 461261,
            "range": "± 8778",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 488361,
            "range": "± 1469",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}