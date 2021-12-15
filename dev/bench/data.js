window.BENCHMARK_DATA = {
  "lastUpdate": 1639611577030,
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
          "id": "a5de219cee8d3ad3d20b20e6eac8b29505bbe296",
          "message": "Merge pull request #258 from zcash/ci-benchmarks\n\nCI: Benchmark tweaks",
          "timestamp": "2021-12-15T23:14:33Z",
          "tree_id": "d85ca12f3d27eeae9f70851715e70d8102637221",
          "url": "https://github.com/zcash/orchard/commit/a5de219cee8d3ad3d20b20e6eac8b29505bbe296"
        },
        "date": 1639611576087,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 7330496120,
            "range": "± 46619902",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 7323503768,
            "range": "± 29905449",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 10684046562,
            "range": "± 25793456",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 14034021160,
            "range": "± 187973267",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 39223436,
            "range": "± 292289",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 39423507,
            "range": "± 589405",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 44452387,
            "range": "± 353444",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 48959710,
            "range": "± 583612",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1267565,
            "range": "± 2782",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 161244,
            "range": "± 450",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1263208,
            "range": "± 2940",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 164112354,
            "range": "± 160396",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 24927181,
            "range": "± 48252",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2845799,
            "range": "± 4742",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 24871493,
            "range": "± 36201",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2800417,
            "range": "± 6706",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 124523213,
            "range": "± 155552",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 14160062,
            "range": "± 20891",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 124283632,
            "range": "± 182288",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 13938440,
            "range": "± 144952",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 249179998,
            "range": "± 469469",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 28287192,
            "range": "± 39099",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 248535582,
            "range": "± 402184",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 27855066,
            "range": "± 54256",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 45282,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 167496,
            "range": "± 526",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 182381,
            "range": "± 659",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 301587,
            "range": "± 854",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 301331,
            "range": "± 709",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 170989,
            "range": "± 427",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 185865,
            "range": "± 462",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 304869,
            "range": "± 867",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 304914,
            "range": "± 856",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 358070,
            "range": "± 1017",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 372950,
            "range": "± 1239",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 492034,
            "range": "± 2038",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 491739,
            "range": "± 1829",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 599309,
            "range": "± 943",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 678443,
            "range": "± 1518",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}