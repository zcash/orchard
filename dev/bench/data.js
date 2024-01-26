window.BENCHMARK_DATA = {
  "lastUpdate": 1706245658705,
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
          "id": "5db6454d385277559adf5da01ccb78af5520bd9e",
          "message": "Merge pull request #415 from zcash/zip32-scope\n\nUse the `zip32::Scope` type",
          "timestamp": "2024-01-25T21:52:18-07:00",
          "tree_id": "cea7a92acd6c5705c92c183876ccd319545619f8",
          "url": "https://github.com/zcash/orchard/commit/5db6454d385277559adf5da01ccb78af5520bd9e"
        },
        "date": 1706245657714,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2905892435,
            "range": "± 36750965",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2888170890,
            "range": "± 6471505",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4137378463,
            "range": "± 31614190",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5386905595,
            "range": "± 31166877",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25252725,
            "range": "± 547491",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 24928061,
            "range": "± 564280",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28125934,
            "range": "± 486084",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31777272,
            "range": "± 485401",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1522999,
            "range": "± 10662",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127620,
            "range": "± 432",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1521072,
            "range": "± 12500",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1353132150,
            "range": "± 995290",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16078917,
            "range": "± 14237",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2166550,
            "range": "± 6479",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16066846,
            "range": "± 53599",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2128006,
            "range": "± 7059",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80337347,
            "range": "± 313152",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10777569,
            "range": "± 34315",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80290591,
            "range": "± 205023",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10593680,
            "range": "± 31329",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 160680414,
            "range": "± 275829",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21551173,
            "range": "± 63531",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160508461,
            "range": "± 3432060",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21172300,
            "range": "± 59813",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 465287,
            "range": "± 1611",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 500403,
            "range": "± 811",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}