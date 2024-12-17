window.BENCHMARK_DATA = {
  "lastUpdate": 1734453257537,
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
          "id": "fe389a1dced213a1db87c89dd88164a9a952b09b",
          "message": "Merge pull request #446 from zcash/no-std\n\nAdd no-std support via a default-enabled `std` feature flag",
          "timestamp": "2024-12-17T09:20:20-07:00",
          "tree_id": "b372aa00e05e3025fe6896ea1c7466dbef88ee34",
          "url": "https://github.com/zcash/orchard/commit/fe389a1dced213a1db87c89dd88164a9a952b09b"
        },
        "date": 1734453256608,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2906535070,
            "range": "± 165823550",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2894422283,
            "range": "± 9198601",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4137628718,
            "range": "± 17517648",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5384053473,
            "range": "± 16320286",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25158409,
            "range": "± 540265",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25133053,
            "range": "± 1750251",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28162130,
            "range": "± 623826",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31729389,
            "range": "± 448934",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1529521,
            "range": "± 17243",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 126891,
            "range": "± 466",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1527301,
            "range": "± 4789",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1339388507,
            "range": "± 3425796",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16152217,
            "range": "± 179205",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2155127,
            "range": "± 16369",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16136114,
            "range": "± 15944",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2119506,
            "range": "± 19800",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80712426,
            "range": "± 137019",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10705146,
            "range": "± 32214",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80652304,
            "range": "± 328733",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10521134,
            "range": "± 30551",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161435034,
            "range": "± 1096492",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21393018,
            "range": "± 45103",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 161309958,
            "range": "± 205319",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21022333,
            "range": "± 134319",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 465720,
            "range": "± 4464",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 502428,
            "range": "± 6033",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}