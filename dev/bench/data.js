window.BENCHMARK_DATA = {
  "lastUpdate": 1703034708218,
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
          "id": "a0f1acadb3fc188eea798bdadac4d3bf1786464c",
          "message": "Merge pull request #408 from daira/fix-changelog-entries\n\nMove some changelog entries from 0.3.0 to 0.4.0",
          "timestamp": "2023-12-20T00:57:34Z",
          "tree_id": "93dfd75729dd33d4474d897d62b16846cfb28ada",
          "url": "https://github.com/zcash/orchard/commit/a0f1acadb3fc188eea798bdadac4d3bf1786464c"
        },
        "date": 1703034705732,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2906739669,
            "range": "± 119116788",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2894530239,
            "range": "± 13884032",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4149217045,
            "range": "± 28061375",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5379350986,
            "range": "± 22000951",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25180705,
            "range": "± 553834",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25339992,
            "range": "± 537439",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28387049,
            "range": "± 476596",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31903999,
            "range": "± 1035781",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1527913,
            "range": "± 14237",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127016,
            "range": "± 416",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1525530,
            "range": "± 8957",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1371644993,
            "range": "± 2487364",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16132719,
            "range": "± 88709",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2159338,
            "range": "± 12639",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16121086,
            "range": "± 51884",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2130643,
            "range": "± 14552",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80593275,
            "range": "± 209769",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10743749,
            "range": "± 32803",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80537172,
            "range": "± 1358838",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10600712,
            "range": "± 39336",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161158524,
            "range": "± 290395",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21469155,
            "range": "± 65294",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 161061697,
            "range": "± 271990",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21190567,
            "range": "± 95699",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 466583,
            "range": "± 1525",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 500495,
            "range": "± 1597",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}