window.BENCHMARK_DATA = {
  "lastUpdate": 1734544716087,
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
          "id": "4fa6d3b549f8803016a309281404eab095d04de8",
          "message": "Merge pull request #449 from zcash/pczt-improvements\n\nPCZT improvements",
          "timestamp": "2024-12-18T10:44:36-07:00",
          "tree_id": "cd5682d1f62278f2e1e66374be1b513f826e8679",
          "url": "https://github.com/zcash/orchard/commit/4fa6d3b549f8803016a309281404eab095d04de8"
        },
        "date": 1734544715147,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2910641046,
            "range": "± 32392481",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2896613205,
            "range": "± 16941569",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4165705511,
            "range": "± 20873931",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5422482006,
            "range": "± 30102336",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25005377,
            "range": "± 445238",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25447661,
            "range": "± 636185",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28081047,
            "range": "± 1086110",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31621686,
            "range": "± 447714",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1526037,
            "range": "± 9056",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 126447,
            "range": "± 554",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1522858,
            "range": "± 7097",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1348006534,
            "range": "± 1556275",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16091217,
            "range": "± 37291",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2148453,
            "range": "± 5451",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16064093,
            "range": "± 41971",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2123624,
            "range": "± 5231",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80420770,
            "range": "± 224196",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10688270,
            "range": "± 29016",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80265238,
            "range": "± 177645",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10566316,
            "range": "± 24709",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 160884768,
            "range": "± 208841",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21366416,
            "range": "± 37660",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160509633,
            "range": "± 309131",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21121283,
            "range": "± 432632",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 465664,
            "range": "± 7205",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 502293,
            "range": "± 856",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}