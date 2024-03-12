window.BENCHMARK_DATA = {
  "lastUpdate": 1710286702146,
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
          "id": "e74879dd0ad0918f4ffe0826e03905cd819981bd",
          "message": "Merge pull request #421 from nuttycom/add_rho_type\n\nAdd a `Rho` type, to distinguish from revealed nullifiers of spent notes.",
          "timestamp": "2024-03-12T23:22:14Z",
          "tree_id": "9da6819c647a45ee00f3b953ec2478a64d6f753a",
          "url": "https://github.com/zcash/orchard/commit/e74879dd0ad0918f4ffe0826e03905cd819981bd"
        },
        "date": 1710286701135,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2900176078,
            "range": "± 48298900",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2897947910,
            "range": "± 17474800",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4153424600,
            "range": "± 21343531",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5435044771,
            "range": "± 23874156",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25442611,
            "range": "± 551606",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25703361,
            "range": "± 517851",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28229571,
            "range": "± 504806",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31851421,
            "range": "± 1199007",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1531568,
            "range": "± 7002",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127509,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1528873,
            "range": "± 12023",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1363203263,
            "range": "± 1045684",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16161497,
            "range": "± 43748",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2177467,
            "range": "± 6942",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16149178,
            "range": "± 52401",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2136989,
            "range": "± 13635",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80777914,
            "range": "± 77495",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10836841,
            "range": "± 19683",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80662364,
            "range": "± 224764",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10633406,
            "range": "± 16383",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161557077,
            "range": "± 432612",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21672285,
            "range": "± 71263",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 161290015,
            "range": "± 367893",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21267787,
            "range": "± 105832",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 466115,
            "range": "± 12738",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 500278,
            "range": "± 1329",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}