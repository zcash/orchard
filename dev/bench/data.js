window.BENCHMARK_DATA = {
  "lastUpdate": 1652225091082,
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
          "id": "de37f1cdbcff53e5ab26a485d058bf8c41bd5626",
          "message": "Merge pull request #328 from zcash/release-0.1.0\n\nRelease 0.1.0",
          "timestamp": "2022-05-11T00:05:04+01:00",
          "tree_id": "324bc3f9556eaaa818ac438fd0b9cc283e17a7c0",
          "url": "https://github.com/zcash/orchard/commit/de37f1cdbcff53e5ab26a485d058bf8c41bd5626"
        },
        "date": 1652225089357,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4857861122,
            "range": "± 71667115",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 4806370052,
            "range": "± 25223232",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 6846933148,
            "range": "± 30191351",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 8894090130,
            "range": "± 32810080",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 41323752,
            "range": "± 855174",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 41205365,
            "range": "± 509068",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 46165883,
            "range": "± 1317441",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 50159316,
            "range": "± 8203884",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1317079,
            "range": "± 4192",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 165717,
            "range": "± 777",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1314046,
            "range": "± 791",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 168781331,
            "range": "± 51810",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 25912834,
            "range": "± 17450",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2923277,
            "range": "± 1651",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 25873279,
            "range": "± 16732",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2879646,
            "range": "± 1851",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 129525905,
            "range": "± 339536",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 14554521,
            "range": "± 7564",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 128390326,
            "range": "± 44923",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 14327243,
            "range": "± 9714",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 258982563,
            "range": "± 118605",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 29075555,
            "range": "± 15369",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 258507780,
            "range": "± 448378",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 28635284,
            "range": "± 6284",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 617257,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 684808,
            "range": "± 432",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}