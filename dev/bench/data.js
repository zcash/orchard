window.BENCHMARK_DATA = {
  "lastUpdate": 1711337453302,
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
          "id": "8cd63b31c4d532a515139e3eba26a614adea34e6",
          "message": "Merge pull request #425 from zcash/release-0.8.0\n\norchard release version 0.8.0",
          "timestamp": "2024-03-25T03:17:11Z",
          "tree_id": "9029decd57aa8bb54e306debe670fff61ff4ef48",
          "url": "https://github.com/zcash/orchard/commit/8cd63b31c4d532a515139e3eba26a614adea34e6"
        },
        "date": 1711337452070,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2891437615,
            "range": "± 37502501",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2882547518,
            "range": "± 7916179",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4140410082,
            "range": "± 21964669",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5383838244,
            "range": "± 15482160",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 24925496,
            "range": "± 568475",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 24935924,
            "range": "± 504032",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28232802,
            "range": "± 551259",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31856855,
            "range": "± 446116",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1522123,
            "range": "± 5074",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127469,
            "range": "± 315",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1518848,
            "range": "± 10206",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1357709801,
            "range": "± 1284613",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16065088,
            "range": "± 44761",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2184632,
            "range": "± 8151",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16044156,
            "range": "± 217794",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2134603,
            "range": "± 6692",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80259051,
            "range": "± 199978",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10862629,
            "range": "± 30225",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80110894,
            "range": "± 229142",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10623427,
            "range": "± 76911",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 160473508,
            "range": "± 251338",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21723456,
            "range": "± 63152",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160215182,
            "range": "± 383354",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21238513,
            "range": "± 73100",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 465090,
            "range": "± 1465",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 500730,
            "range": "± 1594",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}