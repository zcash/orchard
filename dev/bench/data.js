window.BENCHMARK_DATA = {
  "lastUpdate": 1738177644487,
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
          "id": "b1c22c07300db22239235d16dab096e23369948f",
          "message": "Merge pull request #442 from zcash/upgrade_incrementalmerkletree\n\nUpgrade `incrementalmerkletree` to version 0.8.1",
          "timestamp": "2025-01-29T11:54:26-07:00",
          "tree_id": "f61a195de36cf0df788a88dd680d1bcbacf0e7e0",
          "url": "https://github.com/zcash/orchard/commit/b1c22c07300db22239235d16dab096e23369948f"
        },
        "date": 1738177642969,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2861691063,
            "range": "± 27404700",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2844288728,
            "range": "± 16380538",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4091077790,
            "range": "± 21586814",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5324978656,
            "range": "± 19148219",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 24249103,
            "range": "± 275500",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 24292515,
            "range": "± 289446",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 27502028,
            "range": "± 259391",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31055361,
            "range": "± 383196",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1478717,
            "range": "± 11099",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 125493,
            "range": "± 420",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1475217,
            "range": "± 6777",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1336868075,
            "range": "± 1743534",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15625002,
            "range": "± 62330",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2131461,
            "range": "± 29469",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15600006,
            "range": "± 40919",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2096569,
            "range": "± 6322",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 78084090,
            "range": "± 172911",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10598358,
            "range": "± 90527",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 77909677,
            "range": "± 215018",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10425188,
            "range": "± 6759",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 156122852,
            "range": "± 204159",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21177997,
            "range": "± 58081",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 155823065,
            "range": "± 284627",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20834817,
            "range": "± 44171",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 463522,
            "range": "± 1318",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 488923,
            "range": "± 6765",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}