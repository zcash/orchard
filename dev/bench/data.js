window.BENCHMARK_DATA = {
  "lastUpdate": 1704943579132,
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
          "id": "d5fa3e105089337d05f1b4e43535c62fb573b708",
          "message": "Merge pull request #413 from zcash/zcash_spec-0.1\n\nMigrate to `zcash_spec 0.1`",
          "timestamp": "2024-01-11T03:11:36Z",
          "tree_id": "cadb4a3e03a221d38888f66a923b6af249d443f8",
          "url": "https://github.com/zcash/orchard/commit/d5fa3e105089337d05f1b4e43535c62fb573b708"
        },
        "date": 1704943577402,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2902358602,
            "range": "± 35890404",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2891976176,
            "range": "± 8924090",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4127953396,
            "range": "± 8477399",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5374611073,
            "range": "± 37504681",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25124293,
            "range": "± 591931",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25293841,
            "range": "± 631791",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28165889,
            "range": "± 678468",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31822172,
            "range": "± 465633",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1524739,
            "range": "± 33085",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127496,
            "range": "± 1024",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1521792,
            "range": "± 6619",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1357827088,
            "range": "± 4612477",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16100593,
            "range": "± 73440",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2170660,
            "range": "± 8197",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16083968,
            "range": "± 94221",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2136541,
            "range": "± 7065",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80438246,
            "range": "± 219110",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10790022,
            "range": "± 31270",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80348661,
            "range": "± 351165",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10628186,
            "range": "± 156519",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 160918499,
            "range": "± 347253",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21568109,
            "range": "± 58786",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160716801,
            "range": "± 3049581",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21241470,
            "range": "± 69666",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 465399,
            "range": "± 4062",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 500204,
            "range": "± 1320",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}