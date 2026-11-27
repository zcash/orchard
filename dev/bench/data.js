window.BENCHMARK_DATA = {
  "lastUpdate": 1795757153689,
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
          "id": "27db24c9ff94199963b0fd9fe022160f3e3ca8bf",
          "message": "Merge pull request #592 from zcash/zip248-per-action-sighashes\n\nSupport per-action and per-binding sighashes for Orchard-protocol bundles",
          "timestamp": "2026-11-26T22:14:02-07:00",
          "tree_id": "a79107dbe00648ae0d7f19d7bd4fe91c4322a9eb",
          "url": "https://github.com/zcash/orchard/commit/27db24c9ff94199963b0fd9fe022160f3e3ca8bf"
        },
        "date": 1795757152685,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2524626963,
            "range": "± 120266399",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2568545407,
            "range": "± 21554340",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3615070774,
            "range": "± 8293584",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 4709169442,
            "range": "± 122956380",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 19064342,
            "range": "± 323456",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 19193343,
            "range": "± 1260937",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 22177354,
            "range": "± 531085",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 25016753,
            "range": "± 346388",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1163569,
            "range": "± 19318",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 104040,
            "range": "± 632",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1160554,
            "range": "± 13054",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1096427099,
            "range": "± 3719942",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 11711345,
            "range": "± 44012",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 1118569,
            "range": "± 13316",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 11682107,
            "range": "± 111990",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 1086116,
            "range": "± 14067",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 58467296,
            "range": "± 368747",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 5507676,
            "range": "± 40157",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 58297604,
            "range": "± 474722",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 5341867,
            "range": "± 32368",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 116920694,
            "range": "± 489285",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 11000668,
            "range": "± 104031",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 116549544,
            "range": "± 613085",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 10665801,
            "range": "± 28888",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}