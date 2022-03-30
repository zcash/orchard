window.BENCHMARK_DATA = {
  "lastUpdate": 1648652220353,
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
          "id": "420d600f0e8276559c50710faf7730ebab35dbec",
          "message": "Merge pull request #305 from zcash/fvk-scope\n\nAdd explicit scoping for viewing keys and addresses",
          "timestamp": "2022-03-30T08:37:20-06:00",
          "tree_id": "4958705fc0ecef315e6352013db8b2c344659784",
          "url": "https://github.com/zcash/orchard/commit/420d600f0e8276559c50710faf7730ebab35dbec"
        },
        "date": 1648652218361,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4917840817,
            "range": "± 31260145",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 4921804311,
            "range": "± 9149315",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 7040824122,
            "range": "± 12857033",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 9148754323,
            "range": "± 249636411",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 38160815,
            "range": "± 881728",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 38123757,
            "range": "± 1173604",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 42862619,
            "range": "± 791894",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 47351329,
            "range": "± 1144941",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1241244,
            "range": "± 11922",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 158677,
            "range": "± 2011",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1237721,
            "range": "± 4888",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 162511757,
            "range": "± 319249",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 24425649,
            "range": "± 41123",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2802955,
            "range": "± 7587",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 24367163,
            "range": "± 48193",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2760932,
            "range": "± 5009",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 122057834,
            "range": "± 156013",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 13949074,
            "range": "± 27404",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 121718927,
            "range": "± 216970",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 13731449,
            "range": "± 25175",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 244064885,
            "range": "± 304874",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 27877024,
            "range": "± 50129",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 243486269,
            "range": "± 516927",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 27448224,
            "range": "± 57834",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 592371,
            "range": "± 1257",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 652331,
            "range": "± 1701",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}