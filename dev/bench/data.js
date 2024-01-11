window.BENCHMARK_DATA = {
  "lastUpdate": 1704945505460,
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
          "id": "c5dea4e33b75f5f2f55822b6775d146b1aac539b",
          "message": "Merge pull request #414 from zcash/zip32-0.1",
          "timestamp": "2024-01-10T20:43:45-07:00",
          "tree_id": "d53c5d0432631368e8c90e43415546d9f2efdec8",
          "url": "https://github.com/zcash/orchard/commit/c5dea4e33b75f5f2f55822b6775d146b1aac539b"
        },
        "date": 1704945504416,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2904667215,
            "range": "± 108192360",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2879596332,
            "range": "± 9730304",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4132256209,
            "range": "± 24214462",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5379385182,
            "range": "± 30257775",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25075350,
            "range": "± 608803",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 24948050,
            "range": "± 497683",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28172688,
            "range": "± 1340647",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31683692,
            "range": "± 401998",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1532093,
            "range": "± 10441",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127464,
            "range": "± 3295",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1528593,
            "range": "± 7266",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1353401026,
            "range": "± 2875305",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16178649,
            "range": "± 61915",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2177353,
            "range": "± 6525",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16141097,
            "range": "± 313965",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2146940,
            "range": "± 6484",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80824843,
            "range": "± 323705",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10840342,
            "range": "± 28593",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80666255,
            "range": "± 149194",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10680381,
            "range": "± 53596",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161641038,
            "range": "± 323684",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21662445,
            "range": "± 67588",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 161330829,
            "range": "± 299454",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21346084,
            "range": "± 59571",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 466405,
            "range": "± 1468",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 502099,
            "range": "± 2478",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}