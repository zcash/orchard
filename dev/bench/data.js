window.BENCHMARK_DATA = {
  "lastUpdate": 1740096067186,
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
          "id": "0ec587c0290205cca01b51d27a0d02dbb1c687d4",
          "message": "Merge pull request #456 from zcash/bump-zcash_spec\n\nMigrate to `zcash_spec 0.2.1` and `zip32 0.2.0`",
          "timestamp": "2025-02-20T16:48:14-07:00",
          "tree_id": "0d7a23aa67c7d0f798cfe642e3685c8e10c0bda1",
          "url": "https://github.com/zcash/orchard/commit/0ec587c0290205cca01b51d27a0d02dbb1c687d4"
        },
        "date": 1740096066141,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2875432443,
            "range": "± 232968128",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2853167141,
            "range": "± 2909317",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4090667296,
            "range": "± 24682054",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5337398115,
            "range": "± 30819800",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 24234602,
            "range": "± 318575",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 24210649,
            "range": "± 1083336",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 27590273,
            "range": "± 367263",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31164084,
            "range": "± 285639",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1475755,
            "range": "± 3938",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 125493,
            "range": "± 422",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1471402,
            "range": "± 5134",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1323205316,
            "range": "± 2967479",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15576865,
            "range": "± 77085",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2130831,
            "range": "± 28763",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15555082,
            "range": "± 108068",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2095787,
            "range": "± 4111",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 77863825,
            "range": "± 85755",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10602048,
            "range": "± 31225",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 77714358,
            "range": "± 180216",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10427046,
            "range": "± 22494",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 155637433,
            "range": "± 2720722",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21191920,
            "range": "± 43531",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 155380687,
            "range": "± 273613",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20847169,
            "range": "± 96605",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 462619,
            "range": "± 1644",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 488651,
            "range": "± 767",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}