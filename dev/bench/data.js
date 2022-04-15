window.BENCHMARK_DATA = {
  "lastUpdate": 1649997732695,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "daira@jacaranda.org",
            "name": "Daira Hopwood",
            "username": "daira"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a1aa5bb12eee5659e2af5c1a127b64c3b5e640bc",
          "message": "Merge pull request #202 from nuttycom/trivial_cleanup\n\nMinor cleanup addressing issues found while performing review for zcash/zcash#5024",
          "timestamp": "2022-04-15T05:25:20+01:00",
          "tree_id": "84b1e15beb936951c8b70f0745eb3b4199a27f4b",
          "url": "https://github.com/zcash/orchard/commit/a1aa5bb12eee5659e2af5c1a127b64c3b5e640bc"
        },
        "date": 1649997731620,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4149868057,
            "range": "± 42618516",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 4131935273,
            "range": "± 11064003",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 5918876804,
            "range": "± 13201482",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 7706071552,
            "range": "± 25764632",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 32350806,
            "range": "± 349692",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 32398006,
            "range": "± 147789",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 36244756,
            "range": "± 1336065",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 40002932,
            "range": "± 901415",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1039601,
            "range": "± 778",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 132379,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1036475,
            "range": "± 10283",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 136391904,
            "range": "± 46009",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 20436767,
            "range": "± 15513",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2336329,
            "range": "± 944",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 20400803,
            "range": "± 12387",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2300709,
            "range": "± 833",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 102235456,
            "range": "± 118645",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 11634757,
            "range": "± 9001",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 101973278,
            "range": "± 64066",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 11449066,
            "range": "± 3418",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 204453579,
            "range": "± 200777",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 23238859,
            "range": "± 10780",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 203841451,
            "range": "± 69771",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 22880785,
            "range": "± 10630",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 494042,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 538896,
            "range": "± 480",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}