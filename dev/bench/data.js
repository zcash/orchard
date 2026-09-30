window.BENCHMARK_DATA = {
  "lastUpdate": 1790804349995,
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
          "id": "19f6688ba237d73ef938276c5a7e7e36f64aa9a7",
          "message": "Merge pull request #570 from zcash/aes-0.9-fpe-0.7\n\nMigrate to aes 0.9 and fpe 0.7",
          "timestamp": "2026-09-30T15:29:15-06:00",
          "tree_id": "e0d50ee11d95310b00f8b0bd412b41a5164fb054",
          "url": "https://github.com/zcash/orchard/commit/19f6688ba237d73ef938276c5a7e7e36f64aa9a7"
        },
        "date": 1790804348589,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 1893774500,
            "range": "± 3326927",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 1926929266,
            "range": "± 18128012",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 2702525274,
            "range": "± 4415870",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 3519609669,
            "range": "± 14681319",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 15898075,
            "range": "± 81349",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 15852574,
            "range": "± 75922",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 18270338,
            "range": "± 109469",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 20525598,
            "range": "± 202687",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 877437,
            "range": "± 1500",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 78734,
            "range": "± 737",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 873650,
            "range": "± 4273",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 827194132,
            "range": "± 494924",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 8826075,
            "range": "± 242746",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 850837,
            "range": "± 5373",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 8796268,
            "range": "± 16555",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 823725,
            "range": "± 1591",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 44051626,
            "range": "± 904056",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 4186499,
            "range": "± 6400",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 43900818,
            "range": "± 62118",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 4045558,
            "range": "± 56942",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 88125294,
            "range": "± 1906178",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 8348602,
            "range": "± 34953",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 87772005,
            "range": "± 114809",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 8072722,
            "range": "± 14064",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}