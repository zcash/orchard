window.BENCHMARK_DATA = {
  "lastUpdate": 1788970446771,
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
          "id": "f2be3a479837df6583110fd44e124d155ec592ee",
          "message": "Merge pull request #551 from zcash/dw/zizmor-and-pin-actions\n\nSetup Zizmor",
          "timestamp": "2026-09-09T10:02:47-06:00",
          "tree_id": "f62032bb6ddbeaa03e5101b6e324ca930e28c607",
          "url": "https://github.com/zcash/orchard/commit/f2be3a479837df6583110fd44e124d155ec592ee"
        },
        "date": 1788970445877,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2525501826,
            "range": "± 14290268",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2520615880,
            "range": "± 2439763",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3607670412,
            "range": "± 14230430",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 4727263688,
            "range": "± 17844981",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 19280403,
            "range": "± 123965",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 19070266,
            "range": "± 272141",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 21985844,
            "range": "± 108197",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 25082000,
            "range": "± 209582",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1258321,
            "range": "± 5550",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 104034,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1252528,
            "range": "± 14134",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1111186816,
            "range": "± 952599",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 12626999,
            "range": "± 52054",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 1123030,
            "range": "± 13507",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 12603124,
            "range": "± 56917",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 1089689,
            "range": "± 11257",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 63064241,
            "range": "± 106494",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 5535066,
            "range": "± 6779",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 62912278,
            "range": "± 106636",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 5366398,
            "range": "± 30637",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 126073414,
            "range": "± 432277",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 11049446,
            "range": "± 17501",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 125859892,
            "range": "± 249215",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 10710265,
            "range": "± 35555",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 381724,
            "range": "± 1444",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 409801,
            "range": "± 1003",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}