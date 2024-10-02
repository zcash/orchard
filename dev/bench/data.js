window.BENCHMARK_DATA = {
  "lastUpdate": 1727891262847,
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
          "id": "23a167e3972632586dc628ddbdd69d156dfd607b",
          "message": "Merge pull request #438 from zcash/release/orchard-0.10.0\n\nRelease orchard version 0.10.0",
          "timestamp": "2024-10-02T11:33:46-06:00",
          "tree_id": "9d992e16f125248281ea0f6cd8b72b3995b97b30",
          "url": "https://github.com/zcash/orchard/commit/23a167e3972632586dc628ddbdd69d156dfd607b"
        },
        "date": 1727891261885,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2931241014,
            "range": "± 21956088",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2909775850,
            "range": "± 11987810",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4183265909,
            "range": "± 38633985",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5410266152,
            "range": "± 21502570",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 24957436,
            "range": "± 1030616",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 24957062,
            "range": "± 296089",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28332598,
            "range": "± 464461",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 32100587,
            "range": "± 353270",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1536783,
            "range": "± 14611",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 128483,
            "range": "± 1906",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1532742,
            "range": "± 10973",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1371237616,
            "range": "± 1794606",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16213670,
            "range": "± 119520",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2189208,
            "range": "± 5356",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16185685,
            "range": "± 517968",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2145919,
            "range": "± 14775",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 81026459,
            "range": "± 71170",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10888175,
            "range": "± 23361",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80902068,
            "range": "± 178715",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10672140,
            "range": "± 27704",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 162040568,
            "range": "± 255585",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21762844,
            "range": "± 138116",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 161764169,
            "range": "± 145194",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21325460,
            "range": "± 46338",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 470313,
            "range": "± 2231",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 505664,
            "range": "± 1103",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}