window.BENCHMARK_DATA = {
  "lastUpdate": 1790630047576,
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
          "id": "b223d12e8e8dcfa4e242ed33a9cbf3512a8e6fb4",
          "message": "Merge pull request #557 from zcash/tracing-optional\n\nMake `tracing` an optional dependency enabled by `circuit`",
          "timestamp": "2026-09-28T15:03:41-06:00",
          "tree_id": "e3312250a0624a81807d156d383f976ee7ec5872",
          "url": "https://github.com/zcash/orchard/commit/b223d12e8e8dcfa4e242ed33a9cbf3512a8e6fb4"
        },
        "date": 1790630045386,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2075953614,
            "range": "± 17413774",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2075708442,
            "range": "± 3314653",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 2959083520,
            "range": "± 13899825",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 3843483643,
            "range": "± 25265625",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 15946917,
            "range": "± 121798",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 15863764,
            "range": "± 147333",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 18713151,
            "range": "± 370627",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 20853587,
            "range": "± 155346",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1062424,
            "range": "± 6968",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 89309,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1060484,
            "range": "± 8544",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 950439742,
            "range": "± 4533425",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 10697906,
            "range": "± 58612",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 963592,
            "range": "± 1333",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 10672145,
            "range": "± 234546",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 931704,
            "range": "± 23349",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 53413670,
            "range": "± 120918",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 4752089,
            "range": "± 24721",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 53283031,
            "range": "± 62652",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 4590563,
            "range": "± 5753",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 106803734,
            "range": "± 1396998",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 9485334,
            "range": "± 188555",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 106581619,
            "range": "± 217035",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 9164989,
            "range": "± 7659",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 325480,
            "range": "± 6887",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 347506,
            "range": "± 6072",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}