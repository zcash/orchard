window.BENCHMARK_DATA = {
  "lastUpdate": 1706303223189,
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
          "id": "8a2abbb9999af56a6c0f694d410f56f846f95963",
          "message": "Merge pull request #416 from zcash/release-0.7.0\n\nRelease 0.7.0",
          "timestamp": "2024-01-26T20:51:46Z",
          "tree_id": "17a86dbd6290ae094befb1ab0e445965e8b3d6a6",
          "url": "https://github.com/zcash/orchard/commit/8a2abbb9999af56a6c0f694d410f56f846f95963"
        },
        "date": 1706303221950,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2913347996,
            "range": "± 270827809",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2888622435,
            "range": "± 21979928",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4123357763,
            "range": "± 33912113",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5404794648,
            "range": "± 15392312",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25535281,
            "range": "± 580367",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25118523,
            "range": "± 752275",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28367207,
            "range": "± 1262565",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31822696,
            "range": "± 398875",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1530003,
            "range": "± 9610",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127877,
            "range": "± 3175",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1527566,
            "range": "± 7077",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1346531464,
            "range": "± 1937626",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16140482,
            "range": "± 39294",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2161687,
            "range": "± 6215",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16125874,
            "range": "± 21171",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2133461,
            "range": "± 49667",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80655116,
            "range": "± 323703",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10754650,
            "range": "± 36580",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80589315,
            "range": "± 222315",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10611058,
            "range": "± 11756",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161326625,
            "range": "± 1883325",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21483889,
            "range": "± 57166",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 161183056,
            "range": "± 291953",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21199148,
            "range": "± 43559",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 465661,
            "range": "± 2771",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 500931,
            "range": "± 1652",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}