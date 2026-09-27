window.BENCHMARK_DATA = {
  "lastUpdate": 1790528626945,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "jack@zodl.com",
            "name": "Kris Nuttycombe",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "39657365d83dad8cb69ac0d0948847ce08eb9537",
          "message": "Merge pull request #558 from zcash/zeroize-spending-keys\n\nZeroize spending-key material on drop behind a `zeroize` feature",
          "timestamp": "2026-09-27T17:52:27+01:00",
          "tree_id": "df71f940aed8beaed7cedb0a8cce209a843dc93f",
          "url": "https://github.com/zcash/orchard/commit/39657365d83dad8cb69ac0d0948847ce08eb9537"
        },
        "date": 1790528625699,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2539166818,
            "range": "± 107948430",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2526157020,
            "range": "± 11723597",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3616981118,
            "range": "± 6427851",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 4711677164,
            "range": "± 14627804",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 19174746,
            "range": "± 231648",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 19207421,
            "range": "± 128955",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 22210207,
            "range": "± 177038",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 24867634,
            "range": "± 838579",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1266545,
            "range": "± 29048",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 103978,
            "range": "± 318",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1263733,
            "range": "± 12998",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1100774637,
            "range": "± 3780134",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 12739672,
            "range": "± 27984",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 1117504,
            "range": "± 14175",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 12703025,
            "range": "± 137601",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 1081775,
            "range": "± 4905",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 63604194,
            "range": "± 96172",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 5507886,
            "range": "± 8613",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 63458845,
            "range": "± 188474",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 5333018,
            "range": "± 260646",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 127283359,
            "range": "± 1292391",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 11015564,
            "range": "± 37122",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 127041198,
            "range": "± 2874311",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 10647771,
            "range": "± 46283",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 381363,
            "range": "± 8920",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 412401,
            "range": "± 2905",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}